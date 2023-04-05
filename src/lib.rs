#![no_std]

mod memory;
mod rooms_ids;
mod settings;

#[macro_use]
extern crate alloc;
use alloc::{string::String, vec::Vec};
use asr::{watcher::Pair, Process};
use once_cell::sync::Lazy;
use rooms_ids::Level;
use spinning_top::{const_spinlock, Spinlock};

const MAIN_MODULE: &str = "PizzaTower.exe";
const IDLE_TICK_RATE: f64 = 10.0;
const RUNNING_TICK_RATE: f64 = 120.0;

#[derive(Default)]
struct MemoryAddresses {
    main_address: Option<asr::Address>,
    room_id: Option<asr::Address>,
    room_id_names_pointer_array: Option<asr::Address>,
}

#[derive(Default)]
struct MemoryValues {
    room_id: Pair<i32>, // room id int in static memory
    room_name: Pair<String>,
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
    settings: Option<settings::Settings>,
    values: Lazy<MemoryValues>,
    addresses: Lazy<MemoryAddresses>,
    current_level: Level,
    current_level_rooms: Vec<i32>,
    room_counter: u32,
    rooms_tracker: Vec<i32>,
}

impl State {

    fn startup(&mut self) {
        self.settings = Some(settings::Settings::register());
        asr::set_tick_rate(IDLE_TICK_RATE);
        self.started_up = true;
    }

    fn init(&mut self) -> Result<(), &str> {
        self.addresses.main_address = match &self.main_process {
            Some(info) => match info.get_module_address(MAIN_MODULE) {
                Ok(address) => Some(address),
                Err(_) => {
                    return Err("Could not get main module address when refreshing memory values.")
                }
            },
            None => return Err("Process info is not initialized."),
        };

        self.room_name_array_sigscan_start()?;

        asr::set_tick_rate(RUNNING_TICK_RATE);
        Ok(())
    }

    fn update(&mut self) {
        if !self.started_up {
            self.startup();
        }

        match &self.main_process {
            None => {
                self.main_process = Process::attach(MAIN_MODULE);
                if !(self.main_process.is_some() && self.init().is_ok()) {
                    return;
                }
                // early return to never work with a None process
                return;
            }
            Some(process) => {
                // if game is closed detatch and look for it again
                if !process.is_open() {
                    asr::set_tick_rate(IDLE_TICK_RATE);
                    self.main_process = None;
                    return;
                }
            }
        }

        if self.refresh_mem_values().is_err() {
            return;
        }

        // unwrap settings
        let Some(settings) = &self.settings else { return };

        // reset using IL timer
        if self.values.il_timer_seconds.decreased()
            && self.values.il_timer_minutes.current == 0.0
            && self.values.score.current == 0.0
            && asr::timer::state() == asr::timer::TimerState::Running
            && !settings.full_game
        {
            self.room_counter = 0;
            self.rooms_tracker = Vec::new();
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
            if settings.full_game {
                // start the timer in full game runs
                if rooms_ids::entered_hub_start(self.values.room_id.current, self.values.room_id.old) {
                    asr::timer::start();
                }
                // split when in crumbling pizza last room and panic becomes 0
                if self.current_level == Level::F5CrumblingTower
                    && self.values.panic.current == 0.0
                    && self.values.panic.old == 1.0
                {
                    asr::timer::split();
                }
                // split on any level exit from their first room
                if rooms_ids::is_in_hub(self.values.room_id.current, self.current_level)
                    && (rooms_ids::final_room(self.values.room_id.old).is_some() || self.values.room_id.old == 281)
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

                if asr::timer::state() == asr::timer::TimerState::Running && !settings.full_game {
                    asr::timer::reset();
                }
            }

            match rooms_ids::entered_level(self.values.room_id.current, self.current_level) {
                Some(level) => {
                    self.current_level = level;
                    self.current_level_rooms =
                        rooms_ids::get_current_level_rooms(self.current_level);
                    asr::timer::set_variable("Current Level", &format!("{:?}", self.current_level));
                    if !settings.full_game
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
            /*
            // to help with the rooms vecs development, remove someday i guess
            if asr::timer::state() == asr::timer::TimerState::Running {
                self.rooms_tracker.push(self.values.room_id.current);
                asr::print_message("New room entered:");
                asr::print_message(&format!("{:?}", self.rooms_tracker));
            }
            */
        }

        // advanced a room split
        if self.current_level_rooms.len() > (self.room_counter + 1) as usize
            && self.values.room_id.current
                == self.current_level_rooms[(self.room_counter + 1) as usize]
        {
            if !settings.full_game && settings.splits_rooms {
                asr::timer::split();
            }
            self.room_counter += 1;
        }
        asr::timer::set_variable("Room Counter", &format!("{}", self.room_counter));

        // end of level split
        // WAR doesn't have a panic so it will use the old room id method that's off by ~0.21s
        if (self.values.panic.current == 0.0 && self.values.panic.old == 1.0
            || self.values.room_id.old == 610 && self.values.room_id.current == 281)
            && !settings.full_game
        {
            asr::timer::split();
        }

        // igt
        let igt = if settings.full_game {
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
    settings: None,
    values: Lazy::new(Default::default),
    addresses: Lazy::new(Default::default),
    current_level: Level::Hub,
    current_level_rooms: vec![],
    room_counter: 0,
    rooms_tracker: vec![],
    
});

#[no_mangle]
pub extern "C" fn update() {
    LS_CONTROLLER.lock().update();
}
