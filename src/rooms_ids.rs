#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Level {
    Hub,
    F1Tutorial,
    F1JohnGutter,
    F1Pizzascape,
    F1AncientCheese,
    F1BloodsauceDungeon,
    F2OreganoDesert,
    F2Wasteyard,
    F2FunFarm,
    F2FastfoodSaloon,
    F3CrustCove,
    F3GnomeForest,
    F3Golf,
    F3DeepDish9,
    F4ThePigCity,
    F4OhShit,
    F4PeppibotFactory,
    F4Refrigerator,
    F5Pizzascare,
    F5DMAS,
    F5War,
    F5CrumblingTower,
    Pepperman,
    Vigilante,
    Noise,
    Fake,
    PizzaFace,
    ResultsScreen,
}

pub fn get_current_level(room_name: &str) -> Option<Level> {
    match room_name {
        "tower_finalhallway" => Some(Level::F5CrumblingTower),
        x if x.contains("tower_tutorial") => Some(Level::F1Tutorial),
        x if x.contains("tower_") => Some(Level::Hub),
        x if x.contains("entrance_") => Some(Level::F1JohnGutter),
        x if x.contains("medieval_") => Some(Level::F1Pizzascape),
        x if x.contains("ruin_") => Some(Level::F1AncientCheese),
        x if x.contains("dungeon_") => Some(Level::F1BloodsauceDungeon),
        "boss_pepperman" => Some(Level::Pepperman),
        x if x.contains("badland_") => Some(Level::F2OreganoDesert),
        x if x.contains("graveyard_") => Some(Level::F2Wasteyard),
        x if x.contains("farm_") => Some(Level::F2FunFarm),
        x if x.contains("saloon_") => Some(Level::F2FastfoodSaloon),
        "boss_vigilante" => Some(Level::Vigilante),
        x if x.contains("plage_") => Some(Level::F3CrustCove),
        x if x.contains("forest_") => Some(Level::F3GnomeForest),
        x if x.contains("space_") => Some(Level::F3DeepDish9),
        x if x.contains("minigolf_") => Some(Level::F3Golf),
        "boss_noise" => Some(Level::Noise),
        x if x.contains("street_") => Some(Level::F4ThePigCity),
        x if x.contains("industrial_") => Some(Level::F4PeppibotFactory),
        x if x.contains("sewer_") => Some(Level::F4OhShit),
        x if x.contains("freezer_") => Some(Level::F4Refrigerator),
        x if x.contains("boss_fakepep") => Some(Level::Fake),
        x if x.contains("chateau_") => Some(Level::F5Pizzascare),
        x if x.contains("kidsparty_") => Some(Level::F5DMAS),
        x if x.contains("war_") => Some(Level::F5War),
        "boss_pizzaface" => Some(Level::PizzaFace),
        "rank_room" => Some(Level::ResultsScreen),
        _ => None, // where did you go?
    }
}

pub fn get_starting_room<'a>(level: &Level) -> &'a str {

    match level {
        Level::Hub => "nonelol",
        Level::F1Tutorial => "tower_tutorial1",
        Level::F1JohnGutter => "entrance_1",
        Level::F1Pizzascape => "medieval_1",
        Level::F1AncientCheese => "ruin_1",
        Level::F1BloodsauceDungeon => "dungeon_1",
        Level::F2OreganoDesert => "badland_1",
        Level::F2Wasteyard => "graveyard_1",
        Level::F2FunFarm => "farm_2",
        Level::F2FastfoodSaloon => "saloon_1",
        Level::F3CrustCove => "plage_entrance",
        Level::F3GnomeForest => "forest_1",
        Level::F3Golf => "minigolf_1",
        Level::F3DeepDish9 => "space_1",
        Level::F4ThePigCity => "street_intro",
        Level::F4OhShit => "sewer_1",
        Level::F4PeppibotFactory => "industrial_1",
        Level::F4Refrigerator => "freezer_1",
        Level::F5Pizzascare => "chateau_1",
        Level::F5DMAS => "kidsparty_1",
        Level::F5War => "war_1",
        Level::F5CrumblingTower => "tower_finalhallway",
        Level::Pepperman => "boss_pepperman",
        Level::Vigilante => "boss_vigilante",
        Level::Noise => "boss_noise",
        Level::Fake => "boss_fakepep",
        Level::PizzaFace => "boss_pizzaface",
        Level::ResultsScreen => "nonelol",
    }

}


/**
 * Returns true if a key room that should enable the split for the current level (in full game) is received
 */
pub fn full_game_split_unlock_rooms(current_room: &str) -> bool {

    return [
        "tower_tutorial10",
        "entrance_10",
        "medieval_10",
        "ruin_11",
        "dungeon_10",
        "badland_9",
        "graveyard_6",
        "farm_11",
        "saloon_6",
        "plage_cavern2",
        "forest_john",
        "space_9",
        "minigolf_8",
        "street_john",
        "sewer_8",
        "industrial_5",
        "freezer_escape1",
        "chateau_9",
        "kidsparty_john",
        "war_1",
        "boss_pepperman",
        "boss_vigilante",
        "boss_noise",
        "boss_fakepepkey",
        "boss_pizzafacefinale",
    ].contains(&current_room)

}

/**
 * Return true if it receives a room that should trigger a split, usually where the levels end
 */
pub fn full_game_split_rooms(exited_level: &str) -> bool {

    return [
        "tower_tutorial1",
        "entrance_1",
        "medieval_1",
        "ruin_1",
        "dungeon_1" ,
        "badland_1",
        "graveyard_1",
        "farm_2",
        "saloon_1",
        "plage_entrance",
        "forest_1",
        "minigolf_1",
        "space_1",
        "street_intro",
        "sewer_1",
        "industrial_1",
        "freezer_1",
        "chateau_1",
        "kidsparty_1",
        "war_13",
        "boss_pepperman",
        "boss_vigilante",
        "boss_noise",
        "boss_fakepepkey",
        "rank_room",
    ].contains(&exited_level)
    
}

/**
 * used to start the timer in full game
 */
pub fn entered_hub_start(room_current: &str, room_old: &str) -> bool {
    room_current == "tower_entrancehall" && room_old == "Finalintro"
}
