#![no_std]

mod memory;
mod rooms_ids;
mod settings;

#[macro_use]
extern crate alloc;
use alloc::string::{String};
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
    prev_room_split: String,
    split_igt: f64,
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

        // reset using IL timer in hub
        if self.values.il_timer_seconds.decreased()
            && self.values.il_timer_minutes.current == 0.0
            && self.values.score.current == 0.0
            && !settings.full_game
            && asr::timer::state() == asr::timer::TimerState::Running
        {
            asr::timer::reset();
        }

        // start while in the first room of the level
        if self.values.room_name.current == rooms_ids::get_starting_room(&self.current_level)
            && !settings.full_game
            && self.values.il_timer_minutes.current == 0.0
            && self.values.il_timer_seconds.current < 0.2
            && asr::timer::state() != asr::timer::TimerState::Running
        {
            self.prev_room_split = String::new();
            asr::timer::reset();
            asr::timer::start();
        }

        // room change actions
        if self.values.room_name.changed() {
            if let Some(level) = rooms_ids::get_current_level(&self.values.room_name.current){
                self.current_level = level;
            };

            if settings.full_game {
                // start the timer in full game runs
                if rooms_ids::entered_hub_start(&self.values.room_name.current, &self.values.room_name.old) {
                    asr::timer::start();
                }

                // split when in crumbling pizza last room and panic becomes 0
                if self.current_level == Level::Hub
                    && self.values.panic.current == 0.0
                    && self.values.panic.old == 1.0
                {
                    asr::timer::split();
                }

                // split on any level exit from their first room
                if self.current_level == Level::Hub && rooms_ids::full_game_split_rooms(&self.values.room_name.old) {
                    asr::timer::split();
                }

                // pizza face defeated split
                if self.values.room_name.current == "boss_pizzafacehub" && self.values.room_name.old == "boss_pizzafacefinale" {
                    asr::timer::split();
                }
            } 
            // IL actions
            else if !settings.full_game {

                // split for new rooms, doesn't split if you enter a secret or the last room split triggered <3s ago 
                if (self.values.room_name.current != self.prev_room_split 
                    && self.values.room_name.old != self.prev_room_split
                    || self.split_igt + 3.0 < (self.values.main_timer_seconds.current + self.values.main_timer_minutes.current * 60.0))
                    && !self.values.room_name.current.contains("secret")
                    && !self.values.room_name.old.contains("secret")
                    && self.values.il_timer_seconds.current + self.values.il_timer_minutes.current * 60.0 > 1.0
                {
                    asr::timer::split();
                    self.prev_room_split = self.values.room_name.old.clone();
                    self.split_igt = self.values.main_timer_seconds.current + self.values.main_timer_minutes.current * 60.0;
                    asr::timer::set_variable("last split", &self.prev_room_split);
                }
                // secret splits
                else if (self.values.room_name.current.contains("secret")
                    || self.values.room_name.old.contains("secret"))
                    && settings.splits_secrets 
                {
                    asr::timer::split();
                }
            }

            // reset in main menu
            if self.values.room_name.current == "Mainmenu"
                && asr::timer::state() == asr::timer::TimerState::Running
            {
                self.split_igt = 0.0;
                asr::timer::reset();
            }

        }
        asr::timer::set_variable("Current Level Enum", &format!("{:?}", self.current_level));

        // end of level split
        if self.values.panic.current == 0.0 && self.values.panic.old == 1.0 && !settings.full_game 
            && self.values.il_timer_seconds.current + self.values.il_timer_minutes.current * 60.0 > 1.0 {
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
    prev_room_split: String::new(),
    split_igt: 0.0,
});

#[no_mangle]
pub extern "C" fn update() {
    LS_CONTROLLER.lock().update();
}
