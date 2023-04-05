use alloc::string::String;
use asr::{watcher::Pair, Address, Process};
use asr::signature::Signature;
use crate::State;

const SCORE: [u64; 4] = [0x691898, 0x30, 0x180, 0x320];
const IL_TIMER_SECONDS: [u64; 4] = [0x691898, 0x30, 0x880, 0xB0];
const IL_TIMER_MINUTES: [u64; 4] = [0x691898, 0x30, 0x880, 0xC0];
const MAIN_TIMER_SECONDS: [u64; 4] = [0x691898, 0x30, 0x880, 0xD0];
const MAIN_TIMER_MINUTES: [u64; 4] = [0x691898, 0x30, 0x880, 0xE0];
const PAUSE_MENU_OPEN: [u64; 4] = [0x691898, 0x30, 0x2E0, 0x880];
const PANIC: [u64; 4] = [0x691898, 0x30, 0x8C0, 0x6E0];
// kinda useless
const FPS: u64 = 0x8A45BC;

const ROOM_ID_ARRAY_SIG: Signature<13> = Signature::new("74 0C 48 8B 05 ?? ?? ?? ?? 48 8B 04 D0");
const ROOM_ID_SIG: Signature<9> = Signature::new("89 3D ?? ?? ?? ?? 48 3B 1D");

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
    let buf = match process.read_pointer_path64::<[u8; 100]>(main_module_addr.0, pointer_path) {
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

    pub fn room_name_array_sigscan_start(&mut self) -> Result<(), &str> {

        // idk just some random ass number, TODO: do it like og ls when possible, it iterates through the memory pages
        let size = 0x3200000;
        let process = self.main_process.as_ref().ok_or("Could not get process from state struct.")?;
        let main_address = self.addresses.main_address.unwrap_or(Address(0));
        
        // get pointer scan add -> read u32 5 bytes after the result to find offset -> result is add scanned + 9 + offset
        let pointer_to_rooms_array = if let Some(add) = ROOM_ID_ARRAY_SIG.scan_process_range(process, main_address, size) {
            let offset = match process.read::<u32>(Address(add.0 + 0x5)){
                Ok(pointer) => pointer,
                Err(_) => return Err("Could not read offset to find the room names array"),
            };
            Address(add.0 + 0x9 + offset as u64)
        } else {
            return Err("Could not find room ID Pointer")
        };

        self.addresses.room_id_names_pointer_array = match process.read::<u64>(pointer_to_rooms_array) {
            Ok(add) => Some(Address(add)),
            Err(_) => return Err("Could not read the array address"),
        };

        asr::print_message("Room name array signature scan complete.");

        // room id sigscan
        self.addresses.room_id = if let Some(add) = ROOM_ID_SIG.scan_process_range(process, main_address, size) {
            let offset = match process.read::<u32>(Address(add.0 + 0x2)){
                Ok(offset) => {
                    offset
                },
                Err(_) => return Err("Could not read offset to find the room names array"),
            };
            Some(Address(add.0 + 0x6 + offset as u64 - main_address.0))
        } else {
            return Err("Could not find room ID Pointer")
        };

        asr::print_message("Room ID signature scan complete.");

        Ok(())
    }

    pub fn refresh_mem_values(&mut self) -> Result<(), &str> {
        let process = if let Some(process) = self.main_process.as_ref() {
            process
        } else {
            return Err("Process could not be loaded");
        };

        let main_address;
        if self.addresses.main_address.is_some() {
            main_address = self.addresses.main_address.unwrap_or(Address(0)).0;
        } else {
            asr::print_message("Could not load main address");
            return Err("Could not load main address");
        }

        if let Ok(value) =
            process.read_pointer_path64::<i32>(main_address, &[self.addresses.room_id.unwrap_or(Address(0)).0])
        {
            update_pair("Room ID", value, &mut self.values.room_id);
        };

        if let Ok(value) = process.read_pointer_path64::<f64>(main_address, &SCORE) {
            update_pair("Score", value, &mut self.values.score);
        };

        if let Ok(value) =
            process.read_pointer_path64::<f64>(main_address, &MAIN_TIMER_SECONDS)
        {
            update_pair(
                "Main IGT Seconds",
                value,
                &mut self.values.main_timer_seconds,
            );
        };

        if let Ok(value) =
            process.read_pointer_path64::<f64>(main_address, &MAIN_TIMER_MINUTES)
        {
            update_pair(
                "Main IGT Minutes",
                value,
                &mut self.values.main_timer_minutes,
            );
        };

        if let Ok(value) =
            process.read_pointer_path64::<f64>(main_address, &IL_TIMER_SECONDS)
        {
            update_pair("IL IGT Seconds", value, &mut self.values.il_timer_seconds);
        };

        if let Ok(value) =
            process.read_pointer_path64::<f64>(main_address, &IL_TIMER_MINUTES)
        {
            update_pair("IL IGT Minutes", value, &mut self.values.il_timer_minutes);
        };

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
        let curr_room_name_add = process.read::<u64>(Address(self.addresses.room_id_names_pointer_array.unwrap_or(Address(0)).0 + self.values.room_id.current as u64 * 0x8));

        match curr_room_name_add {
            Ok(add) => {
                read_string_and_update_pair(&process, Address(0), &[add], "Current Room", &mut self.values.room_name)
            },
            Err(_) => {
                asr::print_message("Could not read the room address, retrying signature scan...");
                self.room_name_array_sigscan_start()?;
            },
        };

        Ok(())
    }
}