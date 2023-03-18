use asr::{watcher::Pair, Process};
use once_cell::sync::Lazy;
use rooms_ids::Level;
use spinning_top::{const_spinlock, Spinlock};
use std::collections::HashMap;

mod rooms_ids;
mod settings;

const MAIN_MODULE: &str = "PizzaTower.exe";
const ROOM_ID_ADDRESS: u64 = 0x8A4588; // room id int in static memory
const SCORE: [u64; 4] = [0x691898, 0x30, 0x180, 0x320];
const IL_TIMER_SECONDS: [u64; 4] = [0x691898, 0x30, 0x880, 0xB0];
const IL_TIMER_MINUTES: [u64; 4] = [0x691898, 0x30, 0x880, 0xC0];
const MAIN_TIMER_SECONDS: [u64; 4] = [0x691898, 0x30, 0x880, 0xD0];
const MAIN_TIMER_MINUTES: [u64; 4] = [0x691898, 0x30, 0x880, 0xE0];
const PAUSE_MENU_OPEN: [u64; 4] = [0x691898, 0x30, 0x2E0, 0x880];
const PANIC: [u64; 4] = [0x691898, 0x30, 0x8C0, 0x6E0];

// kinda useless
const FPS: u64 = 0x8A45BC;

/**
 * update a pair and display it in the variable view of livesplit
 */
fn update_pair<T: std::fmt::Display + Copy>(variable_name: &str, new_value: T, pair: &mut Pair<T>) {
    asr::timer::set_variable(variable_name, &format!("{new_value}"));
    pair.old = pair.current;
    pair.current = new_value;
}

#[derive(Default)]
struct MemoryValues {
    room_id: Pair<i32>,
    score: Pair<f64>,
    main_timer_minutes: Pair<f64>,
    main_timer_seconds: Pair<f64>,
    il_timer_minutes: Pair<f64>,
    il_timer_seconds: Pair<f64>,
    pause_menu_open: Pair<f64>,
    panic: Pair<f64>,
    fps: Pair<i32>,
}

struct State {
    started_up: bool,
    main_process: Option<Process>,
    main_address: asr::Address,
    settings: Lazy<HashMap<String, bool>>,
    values: Lazy<MemoryValues>,
    current_level: Level,
    current_level_rooms: Vec<i32>,
    room_counter: u32,
    rooms_tracker: Vec<i32>,
}

impl State {
    fn refresh_mem_values(&mut self) -> Result<(), &str> {
        let process = if let Some(process) = self.main_process.as_ref() {
            process
        } else {
            return Err("Process could not be loaded");
        };

        if let Ok(value) =
            process.read_pointer_path64::<i32>(self.main_address.0, &[ROOM_ID_ADDRESS])
        {
            update_pair("Room ID", value, &mut self.values.room_id);
        };

        if let Ok(value) = process.read_pointer_path64::<f64>(self.main_address.0, &SCORE) {
            update_pair("Score", value, &mut self.values.score);
        };

        if let Ok(value) =
            process.read_pointer_path64::<f64>(self.main_address.0, &MAIN_TIMER_SECONDS)
        {
            update_pair(
                "Main IGT Seconds",
                value,
                &mut self.values.main_timer_seconds,
            );
        };

        if let Ok(value) =
            process.read_pointer_path64::<f64>(self.main_address.0, &MAIN_TIMER_MINUTES)
        {
            update_pair(
                "Main IGT Minutes",
                value,
                &mut self.values.main_timer_minutes,
            );
        };

        if let Ok(value) =
            process.read_pointer_path64::<f64>(self.main_address.0, &IL_TIMER_SECONDS)
        {
            update_pair("IL IGT Seconds", value, &mut self.values.il_timer_seconds);
        };

        if let Ok(value) =
            process.read_pointer_path64::<f64>(self.main_address.0, &IL_TIMER_MINUTES)
        {
            update_pair("IL IGT Minutes", value, &mut self.values.il_timer_minutes);
        };

        if let Ok(value) = process.read_pointer_path64::<f64>(self.main_address.0, &PAUSE_MENU_OPEN)
        {
            update_pair("Paused", value, &mut self.values.pause_menu_open);
        };

        if let Ok(value) = process.read_pointer_path64::<f64>(self.main_address.0, &PANIC) {
            update_pair("Panic", value, &mut self.values.panic);
        };

        if let Ok(value) = process.read_pointer_path64::<i32>(self.main_address.0, &[FPS]) {
            update_pair("FPS", value, &mut self.values.fps);
        };

        Ok(())
    }

    fn startup(&mut self) {
        let settings_data = settings::get_settings();

        for setting in settings_data {
            self.settings.insert(
                setting.key.to_string(),
                asr::user_settings::add_bool(
                    setting.key,
                    setting.description,
                    setting.default_value,
                ),
            );
        }

        asr::set_tick_rate(10.0);
        self.started_up = true;
    }

    fn init(&mut self) -> Result<(), &str> {
        self.main_address = match &self.main_process {
            Some(info) => match info.get_module_address(MAIN_MODULE) {
                Ok(address) => address,
                Err(_) => {
                    return Err("Could not get main module address when refreshing memory values.")
                }
            },
            None => return Err("Process info is not initialized."),
        };

        asr::set_tick_rate(120.0);
        Ok(())
    }

    fn update(&mut self) {
        if !self.started_up {
            self.startup();
        }

        if self.main_process.is_none() {
            self.main_process = Process::attach(MAIN_MODULE);
            // early return to never work with a None process
            if !(self.main_process.is_some() && self.init().is_ok()) {
                return;
            }
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

        // reset using IL timer
        if self.values.il_timer_seconds.decreased()
            && self.values.il_timer_minutes.current == 0.0
            && self.values.score.current == 0.0
            && asr::timer::state() == asr::timer::TimerState::Running
            && !self.settings["full_game"]
        {
            self.room_counter = 0;
            self.rooms_tracker = vec![];
            asr::timer::reset();
            if rooms_ids::entered_level(self.values.room_id.current, self.current_level)
                != Some(Level::Hub)
            {
                asr::timer::start();
            }
            return;
        }

        // everything related to room changes
        if self.values.room_id.changed() {
            if self.settings["full_game"] {
                // start the timer in full game runs
                if rooms_ids::entered_hub_start(self.values.room_id.current) {
                    asr::timer::start();
                }
                // split when in crumbling pizza last room and panic becomes 0
                if self.current_level == Level::F5CrumblingTower
                    && self.values.panic.current == 0.0
                    && self.values.panic.old == 1.0
                {
                    asr::timer::split();
                }
                // split on any level exit
                if rooms_ids::is_in_hub(self.values.room_id.current, self.current_level)
                    && self.current_level != Level::Hub
                {
                    asr::timer::split();
                }
                // pizza face defeated split
                if self.values.room_id.current == 787 && self.values.room_id.old == 786 {
                    asr::timer::split();
                }
            }

            // always check if player is in hub
            if self.current_level != Level::F5CrumblingTower
                && rooms_ids::is_in_hub(self.values.room_id.current, self.current_level)
            {
                self.current_level = Level::Hub;
                asr::timer::set_variable("Current Level", &format!("{:?}", self.current_level));

                // TODO: find a way to reset even when we are in the first room of the level besides using the score
                if asr::timer::state() == asr::timer::TimerState::Running
                    && !self.settings["full_game"]
                {
                    asr::timer::reset();
                }
            }

            match rooms_ids::entered_level(self.values.room_id.current, self.current_level) {
                Some(level) => {
                    self.current_level = level;
                    self.current_level_rooms =
                        rooms_ids::get_current_level_rooms(self.current_level);
                    asr::timer::set_variable("Current Level", &format!("{:?}", self.current_level));
                    if !self.settings["full_game"]
                        && self.current_level != Level::Hub
                        && asr::timer::state() != asr::timer::TimerState::Running
                    {
                        self.room_counter = 0;
                        self.rooms_tracker = vec![];
                        asr::timer::reset();
                        asr::timer::start();
                    }
                }
                // unknown room / level, keep the last valid one
                None => {
                    asr::timer::set_variable("Current Level", &format!("{:?}", self.current_level));
                }
            }

            // reset in main menu
            if self.values.room_id.current == 776
                && asr::timer::state() == asr::timer::TimerState::Running
            {
                asr::timer::reset();
            }

            // to help with the rooms vecs development, remove someday i guess
            if asr::timer::state() == asr::timer::TimerState::Running {
                self.rooms_tracker.push(self.values.room_id.current);
                asr::print_message("New room entered:");
                asr::print_message(&format!("{:?}", self.rooms_tracker));
            }
        }

        // advanced a room split
        if self.current_level_rooms.len() > (self.room_counter + 1) as usize
            && self.values.room_id.current
                == self.current_level_rooms[(self.room_counter + 1) as usize]
        {
            if !self.settings["full_game"] && self.settings["splits_rooms"] {
                asr::timer::split();
            }
            self.room_counter += 1;
        }
        asr::timer::set_variable("Room Counter", &format!("{}", self.room_counter));

        // end of level split
        // WAR doesn't have a panic so it will use the old room id method that's off by ~0.21s
        if (self.values.panic.current == 0.0 && self.values.panic.old == 1.0
            || self.values.room_id.old == 610 && self.values.room_id.current == 281)
            && !self.settings["full_game"]
        {
            asr::timer::split();
        }

        // igt
        let igt = if self.settings["full_game"] {
            self.values.main_timer_minutes.current * 60.0 + self.values.main_timer_seconds.current
        } else {
            self.values.il_timer_minutes.current * 60.0 + self.values.il_timer_seconds.current
        };
        asr::timer::set_game_time(asr::time::Duration::seconds_f64(igt));
        asr::timer::pause_game_time();
    }
}

static LS_CONTROLLER: Spinlock<State> = const_spinlock(State {
    started_up: false,
    main_process: None,
    main_address: asr::Address(0),
    settings: Lazy::new(HashMap::new),
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
