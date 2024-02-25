extern crate alloc;

use asr::{
    future::{next_tick, IntoOption},
    print_message,
    settings::Gui,
    time::Duration,
    timer::{self, TimerState},
    watcher::Pair,
    Process,
};
use memory::refresh_mem_values;
use rooms_ids::Level;
use settings::TimerMode;
asr::async_main!(stable);

mod memory;
mod settings;
mod rooms_ids;

#[derive(Default, Copy, Clone)]
struct MemoryAddresses {
    main_address: Option<asr::Address>,
    room_id: Option<asr::Address>,
    room_names: Option<asr::Address>,
    buffer_helper: Option<asr::Address>,
}

#[derive(Default)]
struct MemoryValues {
    game_version: Pair<String>,
    room_id: Pair<i32>,
    room_name: Pair<String>,
    file_seconds: Pair<f64>,
    file_minutes: Pair<f64>,
    level_seconds: Pair<f64>,
    level_minutes: Pair<f64>,
    end_of_level: Pair<bool>,
    boss_hp: Pair<u8>,
}

const MAIN_MODULE: &str = "PizzaTower.exe";

async fn main() {
    
    // startup
    asr::set_tick_rate(240.0);
    let mut settings = settings::Settings::register();

    let mut timer_mode_local = settings.timer_mode;

    loop {
        // check if settings GUI changes
        settings.update();
        if timer_mode_local != settings.timer_mode {
            settings.load_default_settings_for_mode();
            timer_mode_local = settings.timer_mode;
        }

        let process_option = Process::attach(MAIN_MODULE);

        let mut mem_addresses = MemoryAddresses::default();
        let mut mem_values = MemoryValues::default();

        let process;
        match process_option {
            Some(process_found) => {
                process = process_found;
                mem_addresses.main_address = process.get_module_address(MAIN_MODULE).into_option();
            }
            None => {
                next_tick().await;
                continue;
            }
        }

        print_message("Connected to Pizza Tower the pizzapasta game");

        process.until_closes(async {

            // init
            if let Ok(address) = memory::room_id_sigscan_start(&process, mem_addresses) {
                mem_addresses.room_id = Some(address);
            } else {
                mem_addresses.room_id = None;
            }

            if mem_addresses.room_id.is_some() {
                if let Ok(room_id_result) = process.read(mem_addresses.main_address.unwrap_or(asr::Address::default()).value() + mem_addresses.room_id.unwrap().value()) {
                    mem_values.room_id.current = room_id_result
                } else {
                    mem_values.room_id.current = 0;
                    print_message(&format!("Could not read room ID before stall that waits for the game opening. Using {}.", mem_values.room_id.current));
                }
                if mem_values.room_id.current == 0 {
                    print_message("Waiting for the game to start...");
                }
                while mem_values.room_id.current == 0 {
                    if mem_values.room_id.current == 0 {
                        if let Ok(value) = process.read::<i32>(mem_addresses.main_address.unwrap_or(asr::Address::default()).value() + mem_addresses.room_id.unwrap().value()) {
                            mem_values.room_id.current = value;
                        } else {
                            break;
                        }
                    }
                }
            }

            print_message(&format!("Current room:{}", mem_values.room_id.current));

            mem_addresses.room_names = memory::room_name_array_sigscan_start(&process).into_option();
            mem_addresses.buffer_helper = memory::buffer_helper_sigscan_init(&process).into_option();

            // variables declaration for the main loop
            let mut current_level = rooms_ids::Level::Unkown;
            let mut igt_file_secs_calculated: Pair<f64> = Pair::default();
            let mut igt_level_secs_calculated: Pair<f64> = Pair::default();

            let mut ng_plus_offset_seconds: Option<f64> = None;
            let mut iw_offset_seconds: Option<f64> = None;

            let mut enable_full_game_split = false;
            let mut ctop_oob_split = false; // should only happen once per run

            let mut last_room_split_name = String::new();
            let mut last_room_split_time = 0.0;

            loop {

                // makes the livesplit game time frozen, if not used it stutters when the igt stops advancing
                timer::pause_game_time();

                settings.update();
                if timer_mode_local != settings.timer_mode {
                    settings.load_default_settings_for_mode();
                    timer_mode_local = settings.timer_mode;
                }

                if let Err(text) = refresh_mem_values(&process, &mem_addresses, &mut mem_values) {
                    print_message(text);
                    print_message("Exiting main loop and retrying...");
                    break;
                }

                igt_file_secs_calculated.old = igt_file_secs_calculated.current;
                igt_file_secs_calculated.current = mem_values.file_minutes.current * 60.0 + mem_values.file_seconds.current;
                igt_level_secs_calculated.old =  igt_level_secs_calculated.current;
                igt_level_secs_calculated.current = mem_values.level_minutes.current * 60.0 + mem_values.level_seconds.current;

                // update current level and enable full game splits
                if mem_values.room_name.changed() {
                    current_level = rooms_ids::get_current_level(&mem_values.room_name.current, current_level);
                    if !enable_full_game_split {
                        enable_full_game_split = rooms_ids::full_game_split_unlock_rooms(&mem_values.room_name.current);
                    }
                }
                timer::set_variable("Current Level", &format!("{:?}", current_level));

                // offsets for ng+ and iw
                if timer::state() == TimerState::NotRunning {
                    // ng+ offset update
                    if ng_plus_offset_seconds.is_none() && mem_values.room_name.current == "tower_entrancehall" && mem_values.level_minutes.current == 0.0 && mem_values.level_seconds.current < 1.0 {
                        ng_plus_offset_seconds = Some(igt_file_secs_calculated.current);
                    }
                    if ng_plus_offset_seconds.is_some() && (mem_values.room_name.current == "hub_loadingscreen" || mem_values.room_name.current == "Finalintro") {
                        ng_plus_offset_seconds = None;
                    }

                    // iw offset update
                    if iw_offset_seconds.is_none() && current_level == rooms_ids::Level::Hub {
                        iw_offset_seconds = Some(igt_file_secs_calculated.current);
                    }
                    if iw_offset_seconds.is_some() && current_level != rooms_ids::Level::Hub {
                        iw_offset_seconds = None;
                    }
                }

                // game time set
                let game_time_livesplit = match settings.timer_mode {
                    TimerMode::FullGame => igt_file_secs_calculated.current,
                    TimerMode::IL => igt_level_secs_calculated.current,
                    TimerMode::NewGamePlus => igt_file_secs_calculated.current - ng_plus_offset_seconds.unwrap_or(0.0),
                    TimerMode::IW => igt_level_secs_calculated.current - iw_offset_seconds.unwrap_or(0.0),
                };
                timer::set_game_time(Duration::seconds_f64(game_time_livesplit));

                // start
                if settings.start_enable {
                    if settings.start_new_file && mem_values.room_name.current == "tower_entrancehall" && settings.start_new_file && mem_values.room_name.old == "Finalintro" {
                        timer::start();
                    }
                    if settings.start_any_file && mem_values.room_name.current == "tower_entrancehall" && mem_values.room_name.old == "hub_loadingscreen" {
                        timer::start();
                    }
                    if settings.start_new_il && rooms_ids::get_starting_room(&current_level) == mem_values.room_name.current && igt_level_secs_calculated.current > 0.07 && igt_level_secs_calculated.current <= 0.1 {
                        timer::start();
                    }
                    if settings.start_exit_level && mem_values.room_name.changed() && rooms_ids::full_game_split_rooms(&mem_values.room_name.old) && current_level == Level::Hub {
                        timer::start();
                    }
                }

                // reset
                if settings.reset_enable {
                    if settings.reset_new_file && mem_values.room_name.current == "Finalintro" && mem_values.room_name.old != "Finalintro" {
                        timer::reset();
                    }
                    if settings.reset_any_file && mem_values.room_name.changed() && mem_values.room_name.current == "hub_loadingscreen" {
                        timer::reset();
                    }
                    if settings.reset_new_level && igt_level_secs_calculated.decreased() && current_level != Level::Hub {
                        timer::reset();
                    }
                }

                // split
                if settings.splits_enable {

                    // covers any full game split
                    if settings.splits_level_end {

                        // standard level / boss end
                        if mem_values.room_name.changed()
                        && rooms_ids::full_game_split_rooms(&mem_values.room_name.old)
                        && (current_level == Level::Hub || current_level == Level::ResultsScreen)
                        && enable_full_game_split
                        && mem_values.boss_hp.old == 0 {
                            timer::split();
                            enable_full_game_split = false;
                        }

                        // end of the run frame perfect split, technically the last "if" could cover this too but frame perfectly splitting at the end is cooler
                        if mem_values.end_of_level.current && !mem_values.end_of_level.old && mem_values.room_name.current == "tower_entrancehall" {
                            timer::split();
                        }

                        // ctop entering from oob
                        if timer::state() == TimerState::NotRunning && ctop_oob_split {
                            ctop_oob_split = false;
                        }
                        if mem_values.room_name.current == "tower_finalhallway" && mem_values.room_name.old == "tower_5" && !ctop_oob_split {
                            ctop_oob_split = true;
                            timer::split();
                        }
                    }

                    if settings.splits_rooms {

                        if (igt_level_secs_calculated.current - last_room_split_time > 2.0 || mem_values.room_name.current != last_room_split_name) 
                        && (mem_values.room_name.changed() || mem_values.end_of_level.current && mem_values.end_of_level.old) {
                            last_room_split_time = igt_level_secs_calculated.current;
                            last_room_split_name = mem_values.room_name.old.clone();
                            asr::timer::split();
		                }

                    }

                }

                next_tick().await;
            }
        }).await;
    }
}
