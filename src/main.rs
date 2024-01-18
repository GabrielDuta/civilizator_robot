use robotics_lib::energy::Energy;
use robotics_lib::event::events::Event;
use robotics_lib::interface;
use robotics_lib::interface::{debug, Direction, go};
use robotics_lib::runner::{Robot, Runnable, Runner};
use robotics_lib::runner::backpack::BackPack;
use robotics_lib::utils::{go_allowed, LibError};
use robotics_lib::world::coordinates::Coordinate;
use robotics_lib::world::{World, world_generator};
use robotics_lib::world::world_generator::Generator;
use rip_worldgenerator;
use strum::IntoEnumIterator;
use charting_tools;
use charting_tools::charted_coordinate::ChartedCoordinate;
use charting_tools::charted_paths::ChartedPaths;
use charting_tools::charted_world::ChartedWorld;
use charting_tools::charting_bot::ChartingBot;
use charting_tools::ChartingTools;
use robotics_lib::world::tile::Tile;
use rustici_planner::tool::{Destination, Planner, PlannerError, PlannerResult};

use crate::visualizer::{temp_debug, visualize_debug};

mod visualizer;
mod explore_world;

const MAP: usize = 150;
struct MyRobot {
    robot: Robot,
    mode: RobotMode,
    charted_world: ChartedWorld,
    // charting_bot: Option<ChartingBot>,
    visualizer:
}

fn main() {

    // let mut world = rip_worldgenerator::MyWorldGen::new();
    let mut world = rip_worldgenerator::MyWorldGen::new_param(MAP, 1, 1, 1, true, false, 2);

    let mut r = MyRobot::new(200);
    let mut run = Runner::new( Box::new(r), &mut world).unwrap();

    'running : loop {
    // for i in 0..10 {
        run.game_tick();
        //time control here
    }

}

impl Runnable for MyRobot {
    fn process_tick(&mut self, world: &mut World) {

        match self.mode {
            RobotMode::InitMap => {
                let (x, _, _) = interface::debug(self, world);
                println!("\n{:?}", self.get_coordinate());
                visualizer::temp_debug(x);

                self.charted_world.init(world);
                self.mode = RobotMode::Discover;
            }
            RobotMode::Discover => {
                self.explore_world(world);
                println!("\n{:?}", self.get_coordinate());
                visualize_debug(interface::robot_map(world));
            }
            RobotMode::Operation => {
                panic!("Finito");
            }
            RobotMode::Recharge => {
                if self.robot.energy.has_enough_energy(1000) {
                    self.mode = RobotMode::Discover;
                }
            }
        }
    }

    fn handle_event(&mut self, event: Event) {
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

    fn new(i: usize) -> MyRobot {

        /* Charting bot -> now useless
        let res_charting = ChartingTools::tool::<ChartingBot>();
        let mut charting_bot = match res_charting {
            Ok(bot) => {Some(bot)}
            Err(e) => {println!("{e}"); None}
        };
         */

        let res_charting = ChartingTools::tool::<ChartedWorld>();
        let mut charted_world = match res_charting {
            Ok(bot) => {Some(bot)}
            Err(e) => {println!("{e}"); None}
        };
        let mut charted_world = charted_world.unwrap();

        MyRobot {
            robot: Robot::new(),
            mode: RobotMode::InitMap,
            charted_world,
            // charting_bot,
        }

    }

    fn explore_world(&mut self, world: &mut World) {
        explore_world::explore(self, world);
    }

    fn find_all_content_type(&mut self, world: &mut World) {

    }

}



#[derive(PartialEq)]
enum RobotMode {
    InitMap,
    Discover,
    Operation,
    Recharge
}