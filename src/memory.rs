use alloc::string::String;
use asr::{watcher::Pair, Address, Process, signature::Signature};
use crate::State;

const SCORE: [u64; 4] = [0x691898, 0x30, 0x180, 0x320];

const IL_TIMER_SECONDS: [u64; 4] = [0x691898, 0x30, 0x880, 0xB0];
const IL_TIMER_MINUTES: [u64; 4] = [0x691898, 0x30, 0x880, 0xC0];
const MAIN_TIMER_SECONDS: [u64; 4] = [0x691898, 0x30, 0x880, 0xD0];
const MAIN_TIMER_MINUTES: [u64; 4] = [0x691898, 0x30, 0x880, 0xE0];

const PAUSE_MENU_OPEN: [u64; 4] = [0x691898, 0x30, 0x2E0, 0x880];
const PANIC: [u64; 4] = [0x691898, 0x30, 0x8C0, 0x6E0];

// for practice mod 1.4

/* const IL_TIMER_SECONDS: [u64; 4] = [0x691898, 0x30, 0x880, 0x100];
const IL_TIMER_MINUTES: [u64; 4] = [0x691898, 0x30, 0x880, 0x110];
const MAIN_TIMER_SECONDS: [u64; 4] = [0x691898, 0x30, 0x880, 0x120];
const MAIN_TIMER_MINUTES: [u64; 4] = [0x691898, 0x30, 0x880, 0x130]; */

// kinda useless
const FPS: u64 = 0x8A45BC;

// the array with all the room names
const ROOM_ID_ARRAY_SIG: Signature<13> = Signature::new("74 0C 48 8B 05 ?? ?? ?? ?? 48 8B 04 D0");
// the id of the current room the player is on (i32)
const ROOM_ID_SIG: Signature<9> = Signature::new("89 3D ?? ?? ?? ?? 48 3B 1D");

// the signature for the mod to get the speedrun IGTs
const SPEEDRUN_IGT: Signature<56> = Signature::new(
    concat!(
        "00 00 00 00 00 2E B6 40", // 5678
        "?? ?? ?? ?? ?? ?? ?? ??",
        "?? ?? ?? ?? ?? ?? ?? ??", // level igt
        "?? ?? ?? ?? ?? ?? ?? ??",
        "?? ?? ?? ?? ?? ?? ?? ??", // file igt
        "?? ?? ?? ?? ?? ?? ?? ??",
        "00 00 00 00 00 48 93 40"  // 1234
    )
);


/**
 * update a pair and display it in the variable view of livesplit
 */
fn update_pair<T: core::fmt::Display + Copy>(variable_name: &str, new_value: T, pair: &mut Pair<T>) {
    asr::timer::set_variable(variable_name, &format!("{new_value}"));
    pair.old = pair.current;
    pair.current = new_value;
}

/**
 * reads a UTF-8 string from memory to update a Pair<String>, if it fails the Pair is left intact, if successful also displays it in the livesplit variable viewer
 */
fn read_string_and_update_pair(
    process: &Process,
    main_module_addr: asr::Address,
    pointer_path: &[u64],
    variable_name: &str,
    pair: &mut Pair<String>,
) {
    let buf = match process.read_pointer_path64::<[u8; 100]>(main_module_addr.value(), pointer_path) {
        Ok(bytes) => bytes.to_vec(),
        Err(_) => {
            return
        },
    };

    let string_as_bytes = if let Some(array) = buf.split(|byte| *byte == 0).next() {
        array.to_vec()
    } else {
        return;
    };

    let parsed_string;
    if let Ok(string) = String::from_utf8(string_as_bytes) {
        parsed_string = string.splitn(2, '\0').collect::<String>();
    } else {
        return;
    }

    asr::timer::set_variable(variable_name, &parsed_string);
    pair.old = pair.current.clone();
    pair.current = parsed_string;
}

impl State {

    pub fn room_id_sigscan_start(&self) -> Result<asr::Address, &str> {

        let process = self.main_process.as_ref().ok_or("Could not get process from state struct.")?;
        let main_address = self.addresses.main_address.unwrap_or(Address::new(0));

        // room id sigscan
        let mut room_id_address: Option<Address> = None;
        for range in process.memory_ranges().rev() {
            let address = range.address().unwrap().value();
            let size = range.size().unwrap_or_default();

            if let Some(add) = ROOM_ID_SIG.scan_process_range(process, (address, size)) {
                let offset = match process.read::<u32>(Address::new(add.value() + 0x2)) {
                    Ok(offset) => {
                        offset
                    },
                    Err(_) => {
                        asr::print_message("Could not find offset for room id");
                        return Err("Could not read offset to find the room names array");
                    },
                };
                room_id_address = Some(Address::new(add.value() + 0x6 + offset as u64 - main_address.value()));
                break;
            }

        }

        match room_id_address {
            Some(address) => {
                asr::timer::set_variable("Room Id Address", &format!("{:X}", room_id_address.unwrap().value()));
                asr::print_message("Room ID signature scan complete.");
                Ok(address)
            },
            None => {
                asr::print_message("Could NOT complete the room ID scan.");
                Err("Could not find room ID Pointer")
            },
        }
    }

    pub fn room_name_array_sigscan_start(&self) -> Result<asr::Address, &str> {

        let process = self.main_process.as_ref().ok_or("Could not get process from state struct.")?;
        
        let mut pointer_to_rooms_array: Option<Address> = None;
        // get pointer scan add -> read u32 5 bytes after the result to find offset -> result is add scanned + 9 + offset
        for range in process.memory_ranges().rev() {
            let address = range.address().unwrap().value();
            let size = range.size().unwrap_or_default();

            if let Some(add) = ROOM_ID_ARRAY_SIG.scan_process_range(process, (address, size)) {
                let offset = match process.read::<u32>(Address::new(add.value() + 0x5)){
                    Ok(pointer) => pointer,
                    Err(_) => return Err("Could not read offset to find the room names array"),
                };
                pointer_to_rooms_array = Some(Address::new(add.value() + 0x9 + offset as u64));
                break;
            };
        }

        match pointer_to_rooms_array {
            Some(address) => {
                match process.read::<u64>(address) {
                    Ok(add) => {
                        asr::print_message("Room name array signature scan complete.");
                        Ok(Address::new(add))
                    },
                    Err(_) => return Err("Could not read the array address"),
                }
            },
            None => return Err("Could not find signature for room names array"),
        }

    }

    pub fn speedrun_timer_sigscan_init(&self) -> Result<asr::Address, &str> {

        let process = self.main_process.as_ref().ok_or("Could not get process from state struct.")?;

        let mut igt_address: Option<Address> = None;
        for range in process.memory_ranges().rev() {
            let address = range.address().unwrap().value();
            let size = range.size().unwrap_or_default();
            if let Some(address) = SPEEDRUN_IGT.scan_process_range(process, (address, size)) {
                igt_address = Some(address);
                break;
            }
        }

        // this is a direct reference to the speedrun data, finding the scanned address is enough
        if let Some(add) = igt_address {
            asr::timer::set_variable("IGT address", &format!("{:X}", igt_address.unwrap_or(Address::new(0)).value()));
            asr::print_message("IGT sigscan complete");
            Ok(add)
        } else {
            let error_message = "Could not complete the IGT sigscan, using hardcoded path...";
            asr::print_message(error_message);
            return Err(error_message)
        }
    }

    pub fn refresh_mem_values(&mut self) -> Result<(), &str> {
        let process = if let Some(process) = self.main_process.as_ref() {
            process
        } else {
            return Err("Process could not be loaded");
        };

        let main_address;
        if self.addresses.main_address.is_some() {
            main_address = self.addresses.main_address.unwrap_or(Address::new(0)).value();
        } else {
            asr::print_message("Could not load main address");
            return Err("Could not load main address");
        }

        if let Ok(value) =
            process.read::<i32>(Address::new(self.addresses.room_id.unwrap_or(Address::new(0)).value() + main_address))
        {
            update_pair("Room ID", value, &mut self.values.room_id);
        };

        if let Ok(value) = process.read_pointer_path64::<f64>(main_address, &SCORE) {
            update_pair("Score", value, &mut self.values.score);
        };

        // only update if speedrun/frame igt address was found
        if let Some(_) = self.addresses.speedrun_igt_start {

            let il_address = self.addresses.speedrun_igt_start.unwrap_or(Address::new(0)).value() + 0x10;
            let full_game_addres = self.addresses.speedrun_igt_start.unwrap_or(Address::new(0)).value() + 0x20;
            let level_seconds_add = self.addresses.speedrun_igt_start.unwrap_or(Address::new(0)).value() + 0x40;
            let level_minutes_add = self.addresses.speedrun_igt_start.unwrap_or(Address::new(0)).value() + 0x50;
            let game_seconds_add = self.addresses.speedrun_igt_start.unwrap_or(Address::new(0)).value() + 0x60;
            let game_minutes_add = self.addresses.speedrun_igt_start.unwrap_or(Address::new(0)).value() + 0x70;

            if let Ok(value) = process.read::<f64>(Address::new(il_address)) {
                update_pair("Speedrun IGT IL Frames", value, &mut self.values.speedrun_il_frames);
            }

            if let Ok(value) = process.read::<f64>(Address::new(full_game_addres)) {
                update_pair("Speedrun IGT Full Frames", value, &mut self.values.speedrun_main_frames);
            }

            if let Ok(value) = process.read::<f64>(Address::new(game_seconds_add)) {
                update_pair(
                    "Main IGT Seconds",
                    value,
                    &mut self.values.main_timer_seconds,
                );
            };

            if let Ok(value) = process.read::<f64>(Address::new(game_minutes_add)) {
                update_pair(
                    "Main IGT Minutes",
                    value,
                    &mut self.values.main_timer_minutes,
                );
            };

            if let Ok(value) = process.read::<f64>(Address::new(level_seconds_add)) {
                update_pair("IL IGT Seconds", value, &mut self.values.il_timer_seconds);
            };

            if let Ok(value) = process.read::<f64>(Address::new(level_minutes_add)) {
                update_pair("IL IGT Minutes", value, &mut self.values.il_timer_minutes);
            };

        } else {

            // only use hardcoded path if igt sigscan didn't work
            if let Ok(value) = process.read_pointer_path64::<f64>(main_address, &MAIN_TIMER_SECONDS)
            {
                update_pair(
                    "Main IGT Seconds",
                    value,
                    &mut self.values.main_timer_seconds,
                );
            };

            if let Ok(value) = process.read_pointer_path64::<f64>(main_address, &MAIN_TIMER_MINUTES)
            {
                update_pair(
                    "Main IGT Minutes",
                    value,
                    &mut self.values.main_timer_minutes,
                );
            };

            if let Ok(value) = process.read_pointer_path64::<f64>(main_address, &IL_TIMER_SECONDS)
            {
                update_pair("IL IGT Seconds", value, &mut self.values.il_timer_seconds);
            };

            if let Ok(value) = process.read_pointer_path64::<f64>(main_address, &IL_TIMER_MINUTES)
            {
                update_pair("IL IGT Minutes", value, &mut self.values.il_timer_minutes);
            };

        }


        if let Ok(value) = process.read_pointer_path64::<f64>(main_address, &PAUSE_MENU_OPEN)
        {
            update_pair("Paused", value, &mut self.values.pause_menu_open);
        };

        if let Ok(value) = process.read_pointer_path64::<f64>(main_address, &PANIC) {
            update_pair("Panic", value, &mut self.values.panic);
        };

        if let Ok(value) = process.read_pointer_path64::<i32>(main_address, &[FPS]) {
            update_pair("FPS", value, &mut self.values.fps);
        };

        // with the current room as an offset, find its name in the array
        let curr_room_name_add = process.read::<u64>(Address::new(self.addresses.room_id_names_pointer_array.unwrap_or(Address::new(0)).value() + self.values.room_id.current as u64 * 0x8));

        match curr_room_name_add {
            Ok(add) => {
                read_string_and_update_pair(&process, Address::new(0), &[add], "Current Room", &mut self.values.room_name)
            },
            Err(_) => {
                asr::print_message("Could not read the room address, retrying signature scan...");
                if let Ok(address) = self.room_name_array_sigscan_start() {
                    self.addresses.room_id_names_pointer_array = Some(address);
                };
            },
        };

        Ok(())
    }
}