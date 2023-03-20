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
}

pub fn get_current_level_rooms(level: Level) -> Vec<i32> {
    match level {
        Level::Hub => vec![],
        Level::F1Tutorial => vec![788, 789, 790, 791, 792, 793, 794, 795, 796, 797, 788],
        Level::F1JohnGutter => vec![
            24, 25, 26, 27, 28, 30, 33, 35, 36, 38, 36, 35, 33, 30, 32, 28, 27, 26, 25, 24, 758,
            38, 36, 35, 33, 30, 32, 28, 27, 26, 25, 24,
        ],
        Level::F1Pizzascape => vec![
            39, 40, 41, 546, 42, 44, 46, 49, 46, 48, 51, 547, 53, 547, 51, 48, 46, 44, 42, 546, 41,
            40, 39, 53, 547, 51, 48, 46, 44, 42, 546, 41, 40, 39,
        ],
        Level::F1AncientCheese => vec![
            54, 55, 56, 58, 54, 59, 60, 62, 64, 68, 69, 66, 69, 544, 545, 64, 62, 60, 59, 54, 69,
            544, 545, 64, 62, 60, 59, 54,
        ],
        Level::F1BloodsauceDungeon => vec![
            71, 72, 74, 75, 77, 79, 82, 84, 86, 88, 86, 84, 82, 79, 77, 75, 74, 72, 71, 88, 86, 84,
            82, 79, 77, 75, 74, 72, 71,
        ],
        Level::F2OreganoDesert => vec![
            719, 720, 721, 722, 723, 724, 723, 725, 723, 728, 800, 729, 731, 726, 727, 721, 720,
            719, 731, 726, 727, 721, 720, 719,
        ],
        Level::F2Wasteyard => vec![
            123, 124, 126, 129, 130, 550, 551, 131, 551, 126, 124, 133, 134, 135, 801, 718, 123,
            131, 551, 126, 124, 133, 134, 135, 801, 718, 123,
        ],
        Level::F2FunFarm => vec![
            138, 140, 137, 141, 142, 143, 147, 148, 145, 149, 151, 150, 152, 153, 154, 156, 141,
            137, 138, 152, 153, 154, 156, 141, 137, 138,
        ],
        Level::F2FastfoodSaloon => vec![
            687, 688, 689, 690, 691, 692, 693, 694, 695, 694, 696, 697, 695, 692, 691, 688, 687,
            761, 697, 695, 692, 691, 688, 687,
        ],
        Level::F3CrustCove => vec![
            706, 707, 708, 709, 711, 710, 708, 713, 714, 715, 716, 713, 708, 712, 707, 706, 762,
            715, 716, 713, 715, 716, 713, 708, 712, 707, 706,
        ],
        Level::F3GnomeForest => vec![
            194, 195, 196, 554, 772, 554, 555, 556, 199, 200, 202, 701, 702, 204, 205, 206, 207,
            208, 209, 210, 211, 703, 705, 702, 704, 556, 555, 554, 196, 195, 764, 703, 705, 702,
            704, 556, 555, 554, 196, 195, 194,
        ],
        Level::F3Golf => vec![
            229, 230, 231, 233, 237, 233, 231, 230, 229, 766, 237, 238, 239, 240, 237, 233, 231,
            230, 229,
        ],
        Level::F3DeepDish9 => vec![
            241, 242, 361, 362, 361, 363, 364, 366, 367, 368, 365, 369, 371, 370, 241, 765, 368,
            365, 369, 371, 370, 241,
        ],
        Level::F4ThePigCity => vec![
            559, 560, 561, 563, 565, 566, 567, 568, 685, 568, 567, 566, 563, 561, 560, 685, 568,
            567, 566, 563, 561, 560, 559,
        ],
        Level::F4OhShit => vec![
            830, 378, 379, 380, 381, 382, 383, 384, 385, 386, 387, 388, 378, 830, 384, 385, 386,
            387, 388, 378, 830,
        ],
        Level::F4PeppibotFactory => vec![
            672, 673, 674, 675, 831, 675, 674, 673, 672, 768, 831, 675, 674, 673, 672,
        ],
        Level::F4Refrigerator => vec![
            441, 442, 443, 444, 445, 446, 453, 454, 456, 454, 457, 458, 460, 461, 802, 461, 462,
            459, 458, 457, 454, 453, 444, 443, 442, 441, 769, 459, 458, 457, 454, 453, 444, 443,
            442, 441,
        ],
        Level::F5Pizzascare => vec![
            244, 246, 248, 250, 251, 253, 254, 255, 257, 255, 254, 253, 251, 246, 244, 257, 255,
            254, 253, 251, 246, 244,
        ],
        Level::F5DMAS => vec![
            596, 597, 598, 599, 600, 601, 602, 603, 604, 605, 606, 607, 608, 609, 608, 669, 603,
            602, 670, 599, 598, 597, 609, 608, 669, 603, 602, 670, 599, 598, 597, 596,
        ],
        Level::F5War => vec![
            526, 527, 528, 531, 532, 533, 534, 535, 536, 537, 668, 610, 823, 526, 527, 528, 531,
            532, 533, 534, 535, 536, 537, 668, 610,
        ],
        Level::F5CrumblingTower => vec![
            739, 740, 742, 741, 743, 744, 745, 746, 747, 748, 749, 750, 751, 752, 753, 754, 755,
            756, 803, 757,
        ],
        Level::Pepperman => vec![],
        Level::Vigilante => vec![],
        Level::Noise => vec![],
        Level::Fake => vec![],
        Level::PizzaFace => vec![],
    }
}

pub fn entered_level(room_id: i32, current_level: Level) -> Option<Level> {
    match room_id {
        788 => Some(Level::F1Tutorial),
        24 => Some(Level::F1JohnGutter),
        39 => Some(Level::F1Pizzascape),
        54 => Some(Level::F1AncientCheese),
        71 => Some(Level::F1BloodsauceDungeon),
        719 => Some(Level::F2OreganoDesert),
        123 => Some(Level::F2Wasteyard),
        138 => Some(Level::F2FunFarm),
        687 => Some(Level::F2FastfoodSaloon),
        706 => Some(Level::F3CrustCove),
        194 => Some(Level::F3GnomeForest),
        229 => Some(Level::F3Golf),
        241 => Some(Level::F3DeepDish9),
        559 => Some(Level::F4ThePigCity),
        830 => Some(Level::F4OhShit),
        672 => Some(Level::F4PeppibotFactory),
        441 => Some(Level::F4Refrigerator),
        244 => Some(Level::F5Pizzascare),
        596 => Some(Level::F5DMAS),
        526 => Some(Level::F5War),
        739 => Some(Level::F5CrumblingTower),
        513 => Some(Level::Pepperman),
        514 => Some(Level::Vigilante),
        515 => Some(Level::Noise),
        783 => Some(Level::Fake),
        659 => Some(Level::PizzaFace),
        _ => {
            if is_in_hub(room_id, current_level) {
                return Some(Level::Hub);
            }
            // unknown room / level
            None
        }
    }
}

pub fn final_room(room_id: i32) -> Option<Level> {
    match room_id {
        788 => Some(Level::F1Tutorial),
        24 => Some(Level::F1JohnGutter),
        39 => Some(Level::F1Pizzascape),
        54 => Some(Level::F1AncientCheese),
        71 => Some(Level::F1BloodsauceDungeon),
        719 => Some(Level::F2OreganoDesert),
        123 => Some(Level::F2Wasteyard),
        138 => Some(Level::F2FunFarm),
        687 => Some(Level::F2FastfoodSaloon),
        706 => Some(Level::F3CrustCove),
        194 => Some(Level::F3GnomeForest),
        229 => Some(Level::F3Golf),
        241 => Some(Level::F3DeepDish9),
        559 => Some(Level::F4ThePigCity),
        830 => Some(Level::F4OhShit),
        672 => Some(Level::F4PeppibotFactory),
        441 => Some(Level::F4Refrigerator),
        244 => Some(Level::F5Pizzascare),
        596 => Some(Level::F5DMAS),
        610 => Some(Level::F5War),
        757 => Some(Level::F5CrumblingTower),
        513 => Some(Level::Pepperman),
        514 => Some(Level::Vigilante),
        515 => Some(Level::Noise),
        783 => Some(Level::Fake),
        659 => Some(Level::PizzaFace),
        _ => None,
    }
}

/**
 * true if entered a known hub level, or is at the end screen of crumbling tower of pizza
 */
pub fn is_in_hub(room_id: i32, current_level: Level) -> bool {
    let hub_rooms = [
        757, 803, 34, 756, // f1
        752, // f2
        748, // f3
        744, // f4
        740, 782, 828, // f5
    ];

    match current_level {
        // crumbling tower reuses hub rooms so it will return to hub only when the results screen is reached or you go to the hub room where it is entered
        Level::F5CrumblingTower => room_id == 281 || room_id == 782,
        _ => hub_rooms.contains(&room_id),
    }
}

/**
 * used to start the timer in full game
 */
pub fn entered_hub_start(room_id_current: i32, room_id_old: i32) -> bool {
    room_id_current == 757 && room_id_old == 798
}
