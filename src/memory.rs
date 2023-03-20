use asr::watcher::Pair;
use crate::{State};

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

impl State {
    pub fn refresh_mem_values(&mut self) -> Result<(), &str> {
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
}