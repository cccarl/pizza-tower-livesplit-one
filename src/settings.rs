#[derive(asr::Settings)]
pub struct Settings {
    #[default = false]
    /// Full Game Mode
    pub full_game: bool,
    #[default = false]
    /// Start on level exit
    pub start_on_exit: bool,
    #[default = true]
    /// Split on new rooms
    pub splits_rooms: bool,
    #[default = true]
    /// Split on secrets
    pub splits_secrets: bool,
}