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
                    print_message("waiting for game to start");
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

            let mut current_level = rooms_ids::Level::Unkown;
            let mut ng_plus_offset_seconds: Option<f64> = None;
            let mut iw_offset_seconds: Option<f64> = None;

            loop {

                settings.update();
                if timer_mode_local != settings.timer_mode {
                    settings.load_default_settings_for_mode();
                    timer_mode_local = settings.timer_mode;
                }

                if let Err(text) = refresh_mem_values(&process, &mem_addresses, &mut mem_values) {
                    print_message(text);
                }

                if mem_values.room_name.changed() {
                    current_level = rooms_ids::get_current_level(&mem_values.room_name.current, current_level);
                }

                // offsets for ng+ and iw
                if timer::state() == TimerState::NotRunning {
                    // ng+ offset update
                    if ng_plus_offset_seconds.is_none() && mem_values.room_name.current == "tower_entrancehall" && mem_values.level_minutes.current == 0.0 && mem_values.level_seconds.current < 1.0 {
                        ng_plus_offset_seconds = Some(mem_values.file_minutes.current * 60.0 + mem_values.file_seconds.current);
                    }
                    if ng_plus_offset_seconds.is_some() && (mem_values.room_name.current == "hub_loadingscreen" || mem_values.room_name.current == "Finalintro") {
                        ng_plus_offset_seconds = None;
                    }

                    // iw offset update
                    if iw_offset_seconds.is_none() && current_level == rooms_ids::Level::Hub {
                        iw_offset_seconds = Some(mem_values.file_minutes.current * 60.0 + mem_values.file_seconds.current);
                    }
                    if iw_offset_seconds.is_some() && current_level != rooms_ids::Level::Hub {
                        iw_offset_seconds = None;
                    }
                }

                // game time set
                let game_time_seconds = match settings.timer_mode {
                    TimerMode::FullGame => mem_values.file_minutes.current * 60.0 + mem_values.file_seconds.current,
                    TimerMode::IL => mem_values.level_minutes.current * 60.0 + mem_values.level_seconds.current,
                    TimerMode::NewGamePlus => mem_values.file_minutes.current * 60.0 + mem_values.file_seconds.current - ng_plus_offset_seconds.unwrap_or(0.0),
                    TimerMode::IW => mem_values.file_minutes.current * 60.0 + mem_values.file_seconds.current - iw_offset_seconds.unwrap_or(0.0),
                };

                timer::set_game_time(Duration::seconds_f64(game_time_seconds));

                // start
                if settings.start_enable {
                    if settings.start_new_file && mem_values.room_name.current == "tower_entrancehall" && settings.start_new_file && mem_values.room_name.old == "Finalintro" {
                        timer::start();
                    }
                    if settings.start_any_file && mem_values.room_name.current == "tower_entrancehall" && mem_values.room_name.old == "hub_loadingscreen" {
                        timer::start();
                    }
                    if settings.start_new_il && rooms_ids::get_starting_room(&current_level) == mem_values.room_name.current && mem_values.level_minutes.current == 0.0 && mem_values.level_seconds.current < 0.5 {
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
                    // TODO: IL reset, is it possible to avoid the fake splits when restarting using the il timer??
                }

                next_tick().await;
            }
        }).await;
    }
}
