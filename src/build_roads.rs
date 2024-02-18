use std::arch::x86_64::_bittest;
use std::io::stdin;
use charting_tools::charted_coordinate::ChartedCoordinate;
use charting_tools::charted_paths::ChartedPaths;
use charting_tools::ChartingTools;
use Fe2O3::civil_engineer::{CivilEngineerError, CivilEngineerTool};
use robotics_lib::interface;
use robotics_lib::interface::{Direction, DirectionIter, go, put, robot_map, teleport};
use robotics_lib::runner::{Robot, Runnable};
use robotics_lib::utils::LibError;
use robotics_lib::world::tile::{Content, Tile, TileType};
use robotics_lib::world::World;
use rustici_planner::tool::Destination;
use strum::IntoEnumIterator;
use crate::collect_rocks::collect;
use crate::MyRobot;

pub(crate) fn build_road(robot: &mut MyRobot, world: &mut World) {
    let map_robot = interface::robot_map(world);
    let mut charted_path = ChartingTools::tool::<ChartedPaths>().unwrap();

    match map_robot {
        Some(map) => {
            charted_path.init(&map, world);

            let start_coordinate = robot.mst.first().unwrap().start_as_coordinate();
            let destination = ChartedCoordinate(start_coordinate.0, start_coordinate.1);
            walk_to_coord(robot, world, &charted_path, destination);

            let robot_coord = ChartedCoordinate::from(robot.get_coordinate());
            let end_coordinate = robot.mst.first().unwrap().end_as_coordinate();
            let destination = ChartedCoordinate(end_coordinate.0, end_coordinate.1);

            robot.mst.remove(0);

            let best_path=charted_path.shortest_path(robot_coord, destination);

            match best_path  {
                None => {}
                Some((_, mut actions)) => {
                    actions.remove(actions.len() - 1);

                    put_road(robot, world, actions, &charted_path);
                }
            }
        }
        None => {}
    }
}

fn put_road(robot: &mut MyRobot, world: &mut World, actions: Vec<ChartedCoordinate>, charted_paths: &ChartedPaths) {
    let robot_coord = (robot.get_coordinate().get_row(), robot.get_coordinate().get_col());
    let (mut coords, mut directions) = charted_to_coord_dir(world, robot_coord, actions);

    while !coords.is_empty() {
        let res = CivilEngineerTool::make_street(robot, world, directions.clone());

        match res {
            Ok(_) => {
                remove_roads(world, &mut coords, &mut directions);
                if coords.len() > 0 {
                    let c = coords.first().unwrap().clone();
                    let c = ChartedCoordinate::from(c);
                    walk_to_coord(robot, world, charted_paths, c)
                }
            }
            Err(e) => {
                match e {
                    CivilEngineerError::NotEnoughRocks => {
                        scarica(robot, world, &mut coords, &mut directions);
                        collect(robot, world);
                        remove_roads(world, &mut coords, &mut directions);
                        if coords.len() > 0 {
                            let c = coords.first().unwrap().clone();
                            let c = ChartedCoordinate::from(c);
                            walk_to_coord(robot, world, charted_paths, c)
                        }
                    }
                    CivilEngineerError::NotEnoughEnergy => {
                        scarica(robot, world, &mut coords, &mut directions);
                        *robot.get_energy_mut() = rust_and_furious_dynamo::dynamo::Dynamo::update_energy();
                    }
                    CivilEngineerError::PutError => {
                        scarica(robot, world, &mut coords, &mut directions);
                        collect(robot, world);
                        remove_roads(world, &mut coords, &mut directions);
                        if coords.len() > 0 {
                            let c = coords.first().unwrap().clone();
                            let c = ChartedCoordinate::from(c);
                            walk_to_coord(robot, world, charted_paths, c)
                        }
                    }
                    CivilEngineerError::DestroyError => {
                        if scarica(robot, world, &mut coords, &mut directions)
                            && coords.len() > 0 {
                            coords.remove(0);
                            directions.remove(0);
                        }
                        if coords.len() > 0 {
                            let c = coords.first().unwrap().clone();
                            let c = ChartedCoordinate::from(c);
                            walk_to_coord(robot, world, charted_paths, c)
                        }
                    }
                    _ => {
                        scarica(robot, world, &mut coords, &mut directions);
                        let size = coords.len();
                        remove_roads(world, &mut coords, &mut directions);
                        let rocks = *robot.get_backpack().get_contents().get(&Content::Rock(0)).unwrap_or(&0);
                        if rocks == 0usize {
                            collect(robot, world);
                        } else if coords.len() == size {
                            coords.remove(0);
                            directions.remove(0);
                        }
                        if coords.len() > 0 {
                            let c = coords.first().unwrap().clone();
                            let c = ChartedCoordinate::from(c);
                            walk_to_coord(robot, world, charted_paths, c)
                        }
                    }
                }
            }
        }
    }
}

fn scarica(robot: &mut MyRobot, world: &mut World, coords: &mut Vec<(usize, usize)>, directions: &mut Vec<(Direction, usize)>) -> bool {
    let trees = robot.get_backpack().get_contents().get(&Content::Tree(0));
    let bushes = robot.get_backpack().get_contents().get(&Content::Bush(0));
    let fires = robot.get_backpack().get_contents().get(&Content::Fire);
    let mut trees = *trees.unwrap_or(&0usize);
    let mut bushes = *bushes.unwrap_or(&0usize);
    let mut fires = *fires.unwrap_or(&0usize);

    for dir in Direction::iter() {
        if dir == directions.first().unwrap().0 {
            continue;
        }
        if trees >= 20 {
            match put(robot, world, Content::Tree(0), trees, dir.clone()) {
                Ok(x) => {trees -= x; break;}
                Err(e) => {}
            }
        }
        if bushes > 0 {
            match put(robot, world, Content::Bush(bushes), bushes, dir.clone()) {
                Ok(x) => {bushes -= x;}
                Err(e) => {}
            }
        }
        if fires > 0 {
            match put(robot, world, Content::Fire, bushes, dir.clone()) {
                Ok(x) => {
                    fires -= x;
                }
                Err(e) => {}
            }
        }
    }
    let trees2 = *robot.get_backpack().get_contents().get(&Content::Tree(0)).unwrap();
    let bushes2 = *robot.get_backpack().get_contents().get(&Content::Bush(0)).unwrap();
    let fires2 = *robot.get_backpack().get_contents().get(&Content::Fire).unwrap();
    if trees2 == trees && bushes2 == bushes2 && fires2 == fires {
        return true;
    }
    false
}

fn remove_roads(world: &mut World, coords: &mut Vec<(usize, usize)>, directions: &mut Vec<(Direction, usize)>) {
    let map = robot_map(world);
    let mut exit = false;

    match map {
        None => {}
        Some(w) => {
            while !exit && coords.first().is_some() {
                let c = coords.first().unwrap().clone();
                if w[c.0][c.1].as_ref().unwrap().tile_type == TileType::Street {
                    coords.remove(0);
                    directions.remove(0);
                } else {
                    exit = true;
                }
            }
        }
    }
}

fn charted_to_coord_dir(world: &mut World, start: (usize, usize), actions: Vec<ChartedCoordinate>) -> (Vec<(usize, usize)>, Vec<(Direction, usize)>) {
    let mut coords = Vec::new();
    let mut directions = Vec::new();
    let mut prev = ChartedCoordinate::new(start.0, start.1);
    let mut i = 0;
    let mut do_teleport = false;

    while actions.len() > 0 && i < actions.len() - 1 {
        let (next_i, next_j) = (actions[i].0, actions[i].1);

        if can_build_road(world, next_i, next_j) {
            let next_dir = next_direction(prev, actions[i], &mut do_teleport);
            coords.push((next_i, next_j));
            directions.push((next_dir, 1usize));
        }

        prev = actions[i].clone();
        i += 1;
    }

    (coords, directions)
}

fn can_build_road(world: &mut World, i: usize, j: usize) -> bool {
    let map = robot_map(world);

    match map {
        None => {}
        Some(w) => {
            if w[i][j].as_ref().unwrap().tile_type == TileType::Grass ||
               w[i][j].as_ref().unwrap().tile_type == TileType::Hill ||
               w[i][j].as_ref().unwrap().tile_type == TileType::Sand ||
               w[i][j].as_ref().unwrap().tile_type == TileType::Snow {
                return true;
            }
        }
    }

    false
}

fn walk_to_coord(robot: &mut MyRobot, world: &mut World, charted_path: &ChartedPaths, destination: ChartedCoordinate) {
    let robot_coord = ChartedCoordinate::from(robot.get_coordinate());
    let best_path=charted_path.shortest_path(robot_coord, destination);

    match best_path {
        None => {}
        Some((cost, mut actions)) => {
            if cost <= 0 {
                return;
            }
            actions.remove(0);
            while !actions.is_empty() {
                let robot_coord = ChartedCoordinate::from(robot.get_coordinate());
                let dir = actions.first().unwrap().clone();
                let res;
                let mut do_teleport = false;
                let direction = next_direction(robot_coord, dir, &mut do_teleport);

                if do_teleport {
                    res = teleport(robot, world, (dir.0, dir.1));
                } else {
                    res = go(robot, world, direction);
                }

                match res {
                    Err(e) => {
                        match e {
                            LibError::NotEnoughEnergy => {
                                *robot.get_energy_mut() = rust_and_furious_dynamo::dynamo::Dynamo::update_energy();
                            }
                            _ => {println!("{e:?}")}
                        }
                    }
                    Ok(_) => {
                        actions.remove(0);
                    }
                }
            }
        }
    }
}

fn next_direction(robot_coord: ChartedCoordinate, dir: ChartedCoordinate, do_teleport: &mut bool) -> Direction {
    let mut direction: Direction = Direction::Up;

    if dir.0 != robot_coord.0 && dir.1 != robot_coord.1 {
        *do_teleport = true;
    } else if dir.0 < robot_coord.0 {
        direction = Direction::Up;
    } else if dir.0 > robot_coord.0 {
        direction = Direction::Down;
    } else if dir.1 < robot_coord.1 {
        direction = Direction::Left;
    } else {
        direction = Direction::Right;
    }

    direction
}
