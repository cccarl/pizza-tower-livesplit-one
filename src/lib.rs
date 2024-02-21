extern crate alloc;

use asr::{
    future::{next_tick, IntoOption},
    print_message,
    settings::Gui,
    watcher::Pair,
    Process,
};
use memory::refresh_mem_values;
asr::async_main!(stable);

mod memory;
mod settings;
//mod rooms_ids;

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
}

const MAIN_MODULE: &str = "PizzaTower.exe";

async fn main() {
    // startup
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

            loop {

                settings.update();
                if timer_mode_local != settings.timer_mode {
                    settings.load_default_settings_for_mode();
                    timer_mode_local = settings.timer_mode;
                }

                if let Err(text) = refresh_mem_values(&process, &mem_addresses, &mut mem_values) {
                    print_message(text);
                }

                next_tick().await;
            }
        }).await;
    }
}
