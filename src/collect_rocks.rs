use std::collections::VecDeque;
use charting_tools::charted_coordinate::ChartedCoordinate;
use charting_tools::charted_paths::ChartedPaths;
use charting_tools::ChartingTools;
use robotics_lib::interface;
use robotics_lib::interface::{Direction, go, teleport};
use robotics_lib::runner::Runnable;
use robotics_lib::utils::LibError;
use robotics_lib::world::tile::Content;
use robotics_lib::world::World;
use crate::MyRobot;

pub(crate) fn collect(robot: &mut MyRobot, world: &mut World) {

    let mut rocks_op = robot.get_backpack().get_contents().get(&Content::Rock(0));
    let mut rocks = 0usize;
    let mut count = 0;
    match rocks_op {
        None => {}
        Some(size) => {rocks = *size}
    }

    let map = interface::robot_map(world).unwrap();
    let mut charted_path = ChartingTools::tool::<ChartedPaths>().unwrap();
    charted_path.init(&map, world);

    while count < 5 && rocks <= 20 && robot.rocks.len() > 0 {
        count += 1;
        let my_coordinate = ChartedCoordinate::from(robot.get_coordinate());

        let coord = robot.rocks.remove(0);
        let destination = ChartedCoordinate(coord.0, coord.1);

        let best_path = charted_path.shortest_path(my_coordinate, destination);
        match best_path {
            None => {}
            Some((_, path)) => {
                let mut vecdeque = VecDeque::from_iter(path);
                let last_direction = vecdeque.pop_back().unwrap();
                got_to_destination(robot, world, vecdeque);

                let robot_coord = (robot.get_coordinate().get_row(), robot.get_coordinate().get_col());
                let direction;
                if last_direction.0 < robot_coord.0 {
                    direction = Direction::Up;
                } else if last_direction.0 > robot_coord.0 {
                    direction = Direction::Down;
                } else if last_direction.1 < robot_coord.1 {
                    direction = Direction::Left;
                } else {
                    direction = Direction::Right;
                }

                match interface::destroy(robot, world, direction.clone()) {
                    Ok(size) => {rocks += size}
                    Err(e) => {
                        match e {
                            LibError::NotEnoughEnergy => {
                                *robot.get_energy_mut()=rust_and_furious_dynamo::dynamo::Dynamo::update_energy();
                                match interface::destroy(robot, world, direction.clone()) {
                                    Ok(size) => {rocks += size}
                                    Err(_) => {}
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
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

#[cfg(test)]
mod test {
    use robotics_lib::runner::Runner;
    use robotics_lib::world::world_generator::Generator;
    use crate::MyRobot;

    #[test]
    fn testa() {
        let map_size = 50;
        // let mut world = rip_worldgenerator::MyWorldGen::new_param(map_size, 1, 1, 1, true, false, 10);

        // let mut r = MyRobot::new(map_size);
        // let mut run = Runner::new(Box::new(r), &mut world).unwrap();

        // let (world, _) = world.gen();

        // r.explore_world();

    }
}
