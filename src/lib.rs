use asr::{Process, watcher::Pair};
use rooms_ids::Level;
use spinning_top::{Spinlock, const_spinlock};
use once_cell::sync::Lazy;

mod rooms_ids;

const MAIN_MODULE: &str = "PizzaTower.exe";
const ROOM_ID_PATH: [u64; 1] = [0x8A4588]; // room id int in static memory for some reason
const SCORE: [u64; 4] = [0x691898, 0x30, 0x180, 0x320];
// kinda useless
const CURSOR_IN_MAP_X: [u64; 1] = [0x8A46A8];
const CURSOR_IN_MAP_Y: [u64; 1] = [0x8A46AC];
const WEIRD_COUNTER: [u64; 1] = [0x8A4640];
const FPS: [u64; 1] = [0x8A45BC];

fn update_pair<T: std::fmt::Display + Copy>(variable_name: &str, new_value: T, pair: &mut Pair<T>) {
    asr::timer::set_variable(variable_name, &format!("{new_value}"));
    pair.old = pair.current;
    pair.current = new_value;
}

#[derive(Default)]
struct MemoryValues {
    room_id: Pair<i32>,
    score: Pair<f64>,
    cursor_in_map_x: Pair<i32>,
    cursor_in_map_y: Pair<i32>,
    timer: Pair<i32>,
    fps: Pair<i32>,
}

struct State {
    started_up: bool,
    main_process: Option<Process>,
    values: Lazy<MemoryValues>,
    current_level: Level,
    current_level_rooms: Vec<i32>,
    room_counter: u32,
    rooms_tracker: Vec<i32>,
}

impl State {

    fn refresh_mem_values(&mut self) -> Result<(), &str> {

        let pizza_module = match &self.main_process {
            Some(info) => match info.get_module_address(MAIN_MODULE) {
                Ok(address) => address,
                Err(_) => return Err("Could not get steamworks module address when refreshing memory values.")
            },
            None => return Err("Process info is not initialized.")
        };

        let process = self.main_process.as_ref().unwrap();

        // insert read int calls here
        if let Ok(value) = process.read_pointer_path64::<i32>(pizza_module.0, &ROOM_ID_PATH) {
            update_pair("Room ID", value, &mut self.values.room_id);
        };

        if let Ok(value) = process.read_pointer_path64::<f64>(pizza_module.0, &SCORE) {
            update_pair("Score", value, &mut self.values.score);
        };

        if let Ok(value) = process.read_pointer_path64::<i32>(pizza_module.0, &WEIRD_COUNTER) {
            update_pair("Timer", value, &mut self.values.timer);
        };

        if let Ok(value) = process.read_pointer_path64::<i32>(pizza_module.0, &CURSOR_IN_MAP_X) {
            update_pair("Cursor X", value, &mut self.values.cursor_in_map_x);
        };

        if let Ok(value) = process.read_pointer_path64::<i32>(pizza_module.0, &CURSOR_IN_MAP_Y) {
            update_pair("Cursor Y", value, &mut self.values.cursor_in_map_y);
        };
        
        if let Ok(value) = process.read_pointer_path64::<i32>(pizza_module.0, &FPS) {
            update_pair("FPS", value, &mut self.values.fps);
        };

        Ok(())
    }

    fn startup(&mut self) {
        asr::set_tick_rate(10.0);
        self.started_up = true;
    }

    fn init(&mut self) {
        asr::set_tick_rate(120.0);
    }

    fn update(&mut self) {

        if !self.started_up {
            self.startup();
        }

        if self.main_process.is_none() {
            self.main_process = Process::attach(MAIN_MODULE);
            if self.main_process.is_some() {
                self.init();
            }
            // early return to never work with a None process
            return;
        }

        // if game is closed detatch and look for the game again
        if !self.main_process.as_ref().unwrap().is_open() {
            asr::set_tick_rate(10.0);
            self.main_process = None;
            return;
        }

        if self.refresh_mem_values().is_err() {
            return;
        }

        // reset using score
        if self.values.score.current == 0.0 && self.values.score.decreased() && asr::timer::state() == asr::timer::TimerState::Running {
            asr::timer::reset();
            asr::timer::start();
            self.room_counter = 0;
        }
/*
        // 757 hub at the very start
	    if (self.values.room_id.current == 788 || self.values.room_id.current == 24) && self.values.room_id.changed() {
            // delay start 13 frames so that the final split fits with real time 
            sleep(Duration::from_millis(216));
            asr::timer::start();
        }
*/

        /*
        if (self.values.room_id.current == 281 || self.values.room_id.current == self.values.room_id.old + 1) && self.values.room_id.changed() {
            asr::timer::split();
        }
        */

        // detect level enter, which starts the timer
        // TODO: find a way to reset even when we are in the first room of the level besides using the score
        if self.values.room_id.changed() && asr::timer::state() == asr::timer::TimerState::NotRunning {
            asr::print_message(&format!("New room: {}", self.values.room_id.current));
            match rooms_ids::entered_level(self.values.room_id.current) {
                Some(level) => {
                    asr::timer::set_variable("Current Level", &format!("{:?}", self.current_level));
                    self.current_level = level;
                    self.current_level_rooms = rooms_ids::get_current_level_rooms(self.current_level);
                    self.room_counter = 0;
                    self.rooms_tracker = vec![];
                    asr::timer::set_variable("Current Lvl", &format!("{:?}", self.current_level ));
                    asr::timer::reset();
                    asr::timer::start();
                },
                None => {
                    asr::timer::set_variable("Current Level", &format!("{:?}", self.current_level));
                },
            }
        }

        // always check if player is in hub
        if self.values.room_id.changed() && rooms_ids::is_in_hub(self.values.room_id.current) {
            self.current_level = Level::Hub;
            asr::timer::set_variable("Current Level", &format!("{:?}", self.current_level));

            if asr::timer::state() == asr::timer::TimerState::Running {
                asr::timer::reset();
            }
        }     


        // split
        // advanced a room
        asr::timer::set_variable("ROOMS LEN", &format!("{}", self.current_level_rooms.len()));
        if self.current_level_rooms.len() > (self.room_counter + 1) as usize && self.values.room_id.current == self.current_level_rooms[(self.room_counter + 1) as usize] && self.values.room_id.old == self.current_level_rooms[self.room_counter as usize] {
            asr::timer::split();
            self.room_counter += 1;
            asr::timer::set_variable("Room Counter", &format!("{}", self.room_counter));
        }

        // end of level screen
        if self.values.room_id.current == 281 && self.values.room_id.changed() {
            asr::timer::split();
        }

        // to help with the rooms splits, remove someday i guess
        if self.values.room_id.changed() && asr::timer::state() == asr::timer::TimerState::Running {
            self.rooms_tracker.push(self.values.room_id.current);
            asr::print_message(&format!("{:?}", self.rooms_tracker) );
        }
        

    }

}

static LS_CONTROLLER: Spinlock<State> = const_spinlock(State {
    started_up: false,
    main_process: None,
    values: Lazy::new(Default::default),
    current_level: Level::Hub,
    current_level_rooms: vec![],
    room_counter: 0,
    rooms_tracker: vec![],
});


#[no_mangle]
pub extern "C" fn update() {
    LS_CONTROLLER.lock().update();
}
