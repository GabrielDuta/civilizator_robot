use robotics_lib::energy::Energy;
use robotics_lib::event::events::Event;
use robotics_lib::interface;
use robotics_lib::runner::{Robot, Runnable, Runner};
use robotics_lib::runner::backpack::BackPack;
use robotics_lib::world::coordinates::Coordinate;
use robotics_lib::world::{World};
use robotics_lib::world::world_generator::Generator;
use rip_worldgenerator;
use strum::IntoEnumIterator;
use charting_tools;
use charting_tools::charted_map::ChartedMap;
use charting_tools::ChartingTools;
use robotics_lib::interface::robot_map;
use robotics_lib::world::tile::{Content, Tile};
use shared_state::SharedState;
use crate::collect_rocks::collect;
use crate::poi::Connection;
use crate::RobotMode::Build;

mod visualizer2;
mod explore_world;
mod poi;
mod collect_rocks;
mod build_roads;


struct MyRobot {
    robot: Robot,
    map_size: usize,
    mode: RobotMode,
    charted_coords: ChartedMap<Content>,
    visualizer: visualizer::Visualizer,
    rocks: Vec<(usize, usize)>,
    interest: Vec<(usize, usize)>,
    mst: Vec<Connection>,
    first_tick: bool,
    shared_state: shared_state::SharedState
}

fn main() {

    // let mut world = rip_worldgenerator::MyWorldGen::new();
    let map_size = 100;
    let mut world = rip_worldgenerator::MyWorldGen::new_param(map_size, 1, 0, 1, true, false, 10);

    let mut r = MyRobot::new(map_size, shared_state::SharedState::new());
    let mut run = Runner::new( Box::new(r), &mut world).unwrap();


    'running : loop {
    // for i in 0..10 {
        run.game_tick();
        //time control here

    }

}

impl Runnable for MyRobot {
    fn process_tick(&mut self, world: &mut World) {
        if self.first_tick {
            let debug_info = interface::debug(self, world);
            self.shared_state.update_world(
                debug_info.0, debug_info.1, debug_info.2
            );
            self.first_tick = false;
        }

        match self.mode {
            RobotMode::Discover => {
                self.explore_world(world);
                println!("\n{:?}", self.get_coordinate());
                visualizer2::visualize_debug(interface::robot_map(world));
            }
            RobotMode::Operation => {
                self.find_all_content_type(world);
                self.mst = poi::por(self.interest.clone(), world);

                let map = robot_map(world);
                match map {
                    None => {}
                    Some(w) => {
                        if w[4][4].is_some() &&
                            w[self.map_size - 4][self.map_size - 4].is_some() {
                            self.mst.push(Connection {
                                cost: 100,
                                start: (4, 4),
                                end: (self.map_size - 4, self.map_size - 4)
                            });
                        }
                        if w[4][self.map_size - 4].is_some() &&
                            w[self.map_size - 4][4].is_some() {
                            self.mst.push(Connection {
                                cost: 100,
                                start: (4, self.map_size - 4),
                                end: (self.map_size - 4, 4)
                            });
                        }
                        if w[4][4].is_some() &&
                            w[self.map_size - 4][4].is_some() {
                            self.mst.push(Connection {
                                cost: 100,
                                start: (4, self.map_size - 4),
                                end: (self.map_size - 4, 4)
                            });
                        }
                        if w[4][self.map_size - 4].is_some() &&
                            w[self.map_size - 4][self.map_size - 4].is_some() {
                            self.mst.push(Connection {
                                cost: 100,
                                start: (4, self.map_size - 4),
                                end: (self.map_size - 4, self.map_size - 4)
                            });
                        }
                        if w[4][4].is_some() &&
                            w[4][self.map_size - 4].is_some() {
                            self.mst.push(Connection {
                                cost: 100,
                                start: (4, self.map_size - 4),
                                end: (4, self.map_size - 4)
                            });
                        }
                        if w[self.map_size - 4][4].is_some() &&
                            w[self.map_size - 4][self.map_size - 4].is_some() {
                            self.mst.push(Connection {
                                cost: 100,
                                start: (self.map_size - 4, 4),
                                end: (self.map_size - 4, self.map_size - 4)
                            });
                        }


                        // Print the minimum spanning tree
                        println!("Minimum Spanning Tree:");
                        for connection in self.mst.iter() {
                            println!(
                                "{:?} --({})-- {:?}",
                                connection.start, connection.cost, connection.end
                            );
                        }
                    }
                }

                collect(self, world);
                self.mode = Build;
            }
            RobotMode::Build => {
                if !self.mst.is_empty() && !self.rocks.is_empty() {
                    build_roads::build_road(self, world);
                } else {
                    println!("\n{:?}", self.get_coordinate());
                    visualizer2::visualize_debug(interface::robot_map(world));
                    panic!("Finished!");
                }
            }
            RobotMode::Recharge => {
                if self.robot.energy.has_enough_energy(1000) {
                    self.mode = RobotMode::Discover;
                }
            }
        }
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Ready => {}
            Event::Terminated => {}
            Event::TimeChanged(_) => {
                self.shared_state.update_event(event);
            }
            Event::DayChanged(_) => {
                self.shared_state.update_event(event);
            }
            Event::EnergyRecharged(_) => {
                self.shared_state.update_event(event);
            }
            Event::EnergyConsumed(_) => {
                self.shared_state.update_event(event);
            }
            Event::Moved(_, _) => {
                self.shared_state.update_event(event);
            }
            Event::TileContentUpdated(_, _) => {
                self.shared_state.update_event(event);
            }
            Event::AddedToBackpack(_, _) => {
                self.shared_state.update_event(event);
            }
            Event::RemovedFromBackpack(_, _) => {
                self.shared_state.update_event(event);
            }
        }
    }

    fn get_energy(&self) -> &Energy {
       &self.robot.energy
    }

    fn get_energy_mut(&mut self) -> &mut Energy {
        &mut self.robot.energy
    }

    fn get_coordinate(&self) -> &Coordinate {
        &self.robot.coordinate
    }

    fn get_coordinate_mut(&mut self) -> &mut Coordinate {
        &mut self.robot.coordinate
    }

    fn get_backpack(&self) -> &BackPack {
        &self.robot.backpack
    }

    fn get_backpack_mut(&mut self) -> &mut BackPack {
        &mut self.robot.backpack
    }
}

impl MyRobot {

    fn new(map_size: usize, shared_state: SharedState) -> MyRobot {

        let res_charting = ChartingTools::tool::<ChartedMap<Content>>();
        let mut charted_world = match res_charting {
            Ok(bot) => {Some(bot)}
            Err(e) => {println!("{e}"); None}
        };
        let mut charted_world = charted_world.unwrap();

        MyRobot {
            robot: Robot::new(),
            mode: RobotMode::Discover,
            map_size,
            charted_coords: charted_world,
            // charting_bot,
            visualizer: visualizer::Visualizer::new(),
            rocks: Vec::new(),
            interest: Vec::new(),
            mst: vec![],
            first_tick: true,
            shared_state
        }

    }

    fn explore_world(&mut self, world: &mut World) {
        explore_world::explore(self, world);
    }

    fn find_all_content_type(&mut self, world: &mut World) {
        let map = interface::robot_map(world).unwrap();
        for (i, row) in map.iter().enumerate() {
            for (j, col) in row.iter().enumerate() {
                if col.is_some() {
                    match col.clone().unwrap().content {
                        Content::Rock(_) => {self.rocks.push((i, j))}
                        Content::Tree(_) => {}
                        Content::Garbage(_) => {}
                        Content::Fire => {}
                        Content::Coin(_) => {}
                        Content::Bin(_) => {self.interest.push((i, j))}
                        Content::Crate(_) => {self.interest.push((i, j)); println!("Crate at: {} - {}", i, j)}
                        Content::Bank(_) => {self.interest.push((i, j))}
                        Content::Water(_) => {}
                        Content::Market(_) => {self.interest.push((i, j))}
                        Content::Fish(_) => {}
                        Content::Building => {}
                        Content::Bush(_) => {}
                        Content::JollyBlock(_) => {}
                        Content::Scarecrow => {}
                        Content::None => {}
                    }
                }
            }
        }
    }
}

#[derive(PartialEq)]
enum RobotMode {
    Discover,
    Operation,
    Recharge,
    Build
}