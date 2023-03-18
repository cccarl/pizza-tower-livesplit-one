pub struct Setting<'a> {
    pub key: &'a str,
    pub description: &'a str,
    pub default_value: bool,
}

pub fn get_settings<'a>() -> Vec<Setting<'a>> {
    vec![
        Setting {
            key: "full_game",
            description: "Full Game Mode",
            default_value: false,
        },
        Setting {
            key: "splits_rooms",
            description: "Split on new rooms",
            default_value: true,
        },
    ]
}
