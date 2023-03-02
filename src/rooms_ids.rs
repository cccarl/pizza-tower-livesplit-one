#[derive(Clone, Copy, Debug)]
pub enum Level {
    Hub,
    F1Tutorial,
    F1JohnGutter,
    F1Pizzascape,
    F1AncientCheese,
    F1BloodsauceDungeon,
}

pub fn get_current_level_rooms(level: Level) -> Vec<i32> {

    match level {
        Level::Hub => vec![],
        Level::F1Tutorial => vec![788, 789, 790, 791, 792, 793, 794, 795, 796, 797, 788],
        Level::F1JohnGutter => vec![24, 25, 26, 27, 28, 30, 33, 35, 36, 38, 36, 35, 33, 30, 32, 28, 27, 26, 25, 24, 758, 38, 36, 35, 33, 30, 32, 28, 27, 26, 25, 24],
        Level::F1Pizzascape => vec![39, 40, 41, 546, 42, 44, 46, 49, 46, 48, 51, 547, 53, 547, 51, 48, 46, 44, 42, 546, 41, 40, 39, 53, 547, 51, 48, 46, 44, 42, 546, 41, 40, 39],
        Level::F1AncientCheese => vec![54, 55, 56, 58, 54, 59, 60, 62, 64, 68, 69, 66, 69, 544, 545, 64, 62, 60, 59, 54, 69, 544, 545, 64, 62, 60, 59, 54],
        Level::F1BloodsauceDungeon => vec![71, 72, 74, 75, 77, 79, 82, 84, 86, 88, 86, 84, 82, 79, 77, 75, 74, 72, 71, 88, 86, 84, 82, 79, 77, 75, 74, 72, 71],

    }
}

pub fn entered_level(room_id: i32) -> Option<Level> {

    match room_id {
        788 => Some(Level::F1Tutorial),
        24 => Some(Level::F1JohnGutter),
        39 => Some(Level::F1Pizzascape),
        54 => Some(Level::F1AncientCheese),
        71 => Some(Level::F1BloodsauceDungeon),
        _ => None,
    }

}

pub fn is_in_hub(room_id: i32) -> bool {
    let hub_levels = [757, 803, 34, 756];

    return hub_levels.contains(&room_id);
}