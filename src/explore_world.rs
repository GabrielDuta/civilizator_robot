use std::collections::{HashSet, VecDeque};
use charting_tools::charted_coordinate::ChartedCoordinate;
use charting_tools::charted_paths::ChartedPaths;
use charting_tools::ChartingTools;
use robotics_lib::interface;
use robotics_lib::interface::{Direction, go, teleport};
use robotics_lib::runner::Runnable;
use robotics_lib::utils::{LibError};
use robotics_lib::world::tile::Tile;
use robotics_lib::world::World;
use rustici_planner::tool::{Destination, Planner, PlannerError, PlannerResult};
use crate::{MyRobot, RobotMode};

const MAP_EXT: usize = 100;

pub(crate) fn explore(robot: &mut MyRobot, world: &mut World) {
    let mut not_explore: HashSet<(usize, usize)> = HashSet::new();
    while robot.mode != RobotMode::Operation {
        println!("Explored: {}", world.get_discoverable());
        // Get radius to explore
        let new_destination = Destination::explore(robot.get_energy().get_energy_level(), MAP_EXT);
        // Explore radius
        let result = Planner::planner(robot, new_destination, world);

        match result {
            Ok(planner_result) => {
                match planner_result {
                    PlannerResult::MapAllExplored => {
                        robot.mode = RobotMode::Operation;
                    }
                    PlannerResult::RadiusExplored => {
                        // Go to next coordinate to explore
                        next_discover(robot, world, &mut not_explore);
                    }
                    _ => {}
                }
            },
            Err(error) => {
                match error {
                    PlannerError::MaxEnergyReached => {
                        *robot.get_energy_mut()=rust_and_furious_dynamo::dynamo::Dynamo::update_energy();
                    }
                    PlannerError::RestOfMapIsUnreachable => {
                        next_discover(robot, world, &mut not_explore);
                    }
                    PlannerError::RoboticLibError(e) => {
                        match e {
                            LibError::NotEnoughEnergy => {
                                *robot.get_energy_mut()=rust_and_furious_dynamo::dynamo::Dynamo::update_energy();
                            }
                            _ => { println!("{e:?}"); }
                        }
                    }
                    _ => { println!("{error:?}"); }
                }
            }
        }
    }
}

pub(crate) fn next_discover(robot: &mut MyRobot, world: &mut World, not_explore: &mut HashSet<(usize, usize)>) {
    let map_robot = interface::robot_map(world);

    match map_robot {
        Some(map) => {
            let mut charted_path = ChartingTools::tool::<ChartedPaths>().unwrap();
            charted_path.init(&map, world);
            let my_coordinate = ChartedCoordinate::from(robot.get_coordinate());

            for (i, row) in map.iter().enumerate() {
                for (j, col) in row.iter().enumerate() {
                    if not_explore.contains(&(i, j)) {
                        continue;
                    }
                    match col {
                        None => {}
                        Some(_) => {
                            match can_explore(&map, (i, j), robot.map_size) {
                                Ok(_) => {
                                    let coord = (i, j);
                                    let destination = ChartedCoordinate(coord.0, coord.1);
                                    let best_path=charted_path.shortest_path(my_coordinate,destination);
                                    match best_path {
                                        None => {}
                                        Some((_, path)) => {
                                            got_to_destination(robot, world, VecDeque::from_iter(path));
                                            return;
                                        }
                                    }
                                }
                                Err(_) => {
                                    not_explore.insert((i, j));
                                }
                            }
                        }
                    }
                }
            }
        }
        None => {}
    }
    robot.mode = RobotMode::Operation;
}

fn got_to_destination(robot: &mut MyRobot, world: &mut World, mut actions: VecDeque<ChartedCoordinate>) {
    actions.pop_front();
    while !actions.is_empty() {
        let robot_coord = (robot.get_coordinate().get_row(), robot.get_coordinate().get_col());
        let dir = actions.front().unwrap();
        let mut direction: Direction = Direction::Up;
        let mut do_teleport: bool = false;
        let res;

        if dir.0 != robot_coord.0 && dir.1 != robot_coord.1 {
            do_teleport = true;
        } else if dir.0 < robot_coord.0 {
            direction = Direction::Up;
        } else if dir.0 > robot_coord.0 {
            direction = Direction::Down;
        } else if dir.1 < robot_coord.1 {
            direction = Direction::Left;
        } else {
            direction = Direction::Right;
        }

        if do_teleport {
            res =  teleport(robot, world, (dir.0, dir.1));
        } else {
            res = go(robot, world, direction);
        }

        match res {
            Err(e) => {
                match e {
                    LibError::NotEnoughEnergy => {
                        *robot.get_energy_mut()=rust_and_furious_dynamo::dynamo::Dynamo::update_energy();
                    }
                    _ => {}
                }
            }
            Ok(_) => {
                actions.pop_front();
            }
        }
    }
}

fn can_explore(map: &Vec<Vec<Option<Tile>>>, coord: (usize, usize), map_size: usize) -> Result<(), ()> {
    let (i, j) = (coord.0, coord.1);

    if map[i][j].clone().unwrap().tile_type.properties().walk() {
        if i > 0 && map[i - 1][j].is_none() {
            return Ok(());
        }
        if j + 1 < map_size && map[i][j + 1].is_none() {
            return Ok(());
        }
        if i + 1 < map_size && map[i + 1][j].is_none() {
            return Ok(());
        }
        if j > 0 && map[i][j - 1].is_none() {
            return Ok(());
        }
    }

    Err(())
}
