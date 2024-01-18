use robotics_lib::world::tile::{Content, Tile, TileType};

/*
pub fn debug_world(r: &mut MyRobot, world: &mut World) {
    let (mut tiles, _, _) = debug(r, world);
    let mut tile_def: Vec<Vec<Option<Tile>>>;
    tile_def = Vec::new();

    for (i, row) in tiles.iter().enumerate() {
        tile_def.push(Vec::new());
        for col in row.iter() {
            tile_def[i].push(Some(col.clone()));
        }
    }
}
 */

pub fn temp_debug(ve: Vec<Vec<Tile>>) {
    let mut tile_def: Vec<Vec<Option<Tile>>>;
    tile_def = Vec::new();

    for (i, row) in ve.iter().enumerate() {
        tile_def.push(Vec::new());
        for col in row.iter() {
            tile_def[i].push(Some(col.clone()));
        }
    }

    visualize_debug(Some(tile_def));
}


// Official visualize function
pub fn visualize_debug(tiles: Option<Vec<Vec<Option<Tile>>>>) {
    match tiles {
        None => {println!("Error in getting the world")}
        Some(t) => {
            print_cols(t[0].len());

            for (i, row) in t.iter().enumerate() {
                print!("{:<3}|", i);

                for col in row.iter() {
                    let converted_tile = convert_tile(col);
                    print!("{} ", converted_tile);
                }
                println!();
            }
        }
    }
}

fn convert_tile(tile: &Option<Tile>) -> &str {
    match tile {
        None => "  ",
        Some(t) => {
            return if t.content == Content::None {
                match t.tile_type {
                    TileType::DeepWater => "🔷",
                    TileType::ShallowWater => "🔵",
                    TileType::Sand => "🔶",
                    TileType::Grass => "🌳",
                    TileType::Street => "🛣️",
                    TileType::Hill => "🌱",
                    TileType::Mountain => "⛰️",
                    TileType::Snow => "❄️",
                    TileType::Lava => "🌋",
                    TileType::Teleport(bool) => "🚪",
                    TileType::Wall => "🧱",
                }
            } else {
                match t.content {
                    Content::Rock(_) => "🪨",
                    Content::Tree(_) => "🌲",
                    Content::Garbage(_) => "x",
                    Content::Fire => "🔥",
                    Content::Coin(_) => "🪙",
                    Content::Bin(_) => "🗑️",
                    Content::Crate(_) => "🎭",
                    Content::Bank(_) => "🏦",
                    Content::Water(_) => "🌊",
                    Content::Market(_) => "🏪",
                    Content::Fish(_) => "🐟",
                    Content::Building => "🏠",
                    Content::Bush(_) => "🌱",
                    Content::JollyBlock(_) => "x",
                    Content::Scarecrow => "🐦‍",
                    Content::None => "  ",
                }
            }
        }
    }
}


fn print_cols(len: usize) {
    for i in 0..len {
        print!("  {i}");
    }
    println!();
    for i in 0..(len + 1) {
        print!("___");
    }
    println!();
}
