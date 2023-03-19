#[derive(asr::Settings)]
pub struct Settings {
    #[default = false]
    /// Full Game Mode
    pub full_game: bool,
    #[default = true]
    /// Split on new rooms
    pub splits_rooms: bool,
}