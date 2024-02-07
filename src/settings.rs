use asr::settings::Gui;
use asr::settings::gui::Title;

#[derive(Gui)]
enum TimerMode {
    /// Full Game
    #[default]
    FullGame,
    /// Individual Level
    IL,
    /// New Game+
    NewGamePlus,
    /// Individual World
    IW,
}

#[derive(Gui)]
pub struct Settings {

    /// LiveSplit Timer Mode
    _igt_mode: Title,

    /// Pick a Mode
    timer_mode: TimerMode,

    /// Individual Mode Settings
    _misc: Title,

    #[default = true]
    /// Split on new rooms
    pub splits_rooms: bool,

    #[default = true]
    /// Split on secrets
    pub splits_secrets: bool,

    /// Reminder: Use "-livesplit" in the launch options to use game time.
    _message: Title,
}