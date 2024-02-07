extern crate alloc;

use asr::{future::{next_tick, IntoOption}, print_message, settings::Gui, Process};
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
    room_id: Option<i32>,
}

const MAIN_MODULE: &str = "PizzaTower.exe";


async fn main() {

    // startup
    let mut settings = settings::Settings::register();

    loop {
        let process = Process::wait_attach(MAIN_MODULE).await;

        let mut mem_addresses = MemoryAddresses::default();

        let mut mem_values = MemoryValues::default();

        match process.get_module_address(MAIN_MODULE) {
            Ok(address) => mem_addresses.main_address = Some(address),
            Err(_) => {
                print_message("Could not get address of main module from process, aborting.");
                return
            },
        }

        print_message("Connected to Pizza Tower the pizzapasta game");

        process.until_closes(async {

            // init
            print_message("This only runs once.");

            if let Ok(address) = memory::room_id_sigscan_start(&process, mem_addresses) {
                mem_addresses.room_id = Some(address);
            } else {
                mem_addresses.room_id = None;
            }

            
            if mem_addresses.room_id.is_some() {
                mem_values.room_id = process.read(mem_addresses.main_address.unwrap_or(asr::Address::default()).value() + mem_addresses.room_id.unwrap().value()).into_option();
                while mem_values.room_id.as_ref().unwrap_or(&-1) == &0 {
                    print_message("waiting for game to start");
                    if mem_values.room_id.as_ref().unwrap_or(&-1) == &0 {
                        if let Ok(value) = process.read::<i32>(mem_addresses.main_address.unwrap_or(asr::Address::default()).value() + mem_addresses.room_id.unwrap().value()) {
                            mem_values.room_id = Some(value);
                        } else {
                            break;
                        }
                    }
                }
            }

            print_message(&format!("Current room:{}", mem_values.room_id.unwrap_or(-1)));

            mem_addresses.room_names = memory::room_name_array_sigscan_start(&process).into_option();
            mem_addresses.buffer_helper = memory::buffer_helper_sigscan_init(&process).into_option();


            // TODO: if any of the mem values gave an error, don't enter loop
            loop {
                settings.update();

                

                next_tick().await;
            }
        }).await;
    }

}
