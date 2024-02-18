use std::cmp::Reverse;
use robotics_lib::world::World;
use priority_queue::PriorityQueue;

struct MergeFindSet {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl MergeFindSet {
    fn new(size: usize) -> Self {
        let parent: Vec<usize> = (0..size).collect();
        let rank = vec![0; size];
        MergeFindSet { parent, rank }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, x: usize, y: usize) {
        let root_x = self.find(x);
        let root_y = self.find(y);

        if root_x != root_y {
            if self.rank[root_x] < self.rank[root_y] {
                self.parent[root_x] = root_y;
            } else if self.rank[root_x] > self.rank[root_y] {
                self.parent[root_y] = root_x;
            } else {
                self.parent[root_x] = root_y;
                self.rank[root_y] += 1;
            }
        }
    }
}


#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Connection {
    pub cost: usize,
    pub start: (usize, usize),
    pub end: (usize, usize),
}

impl Connection {
    pub fn start_as_coordinate(&self) -> (usize, usize) {
        self.start
    }

    pub fn end_as_coordinate(&self) -> (usize, usize) {
        self.end
    }
}

fn kruskal(coordinates: Vec<(usize, usize)>, connections: Vec<Connection>) -> Vec<Connection> {
    let mut result = Vec::new();
    let mut mf_set = MergeFindSet::new(coordinates.len());
    let mut sorted_connections: Vec<Connection> = connections.into_iter().collect();

    // Sort the connections in ascending order based on cost
    sorted_connections.sort_by(|a, b| a.cost.cmp(&b.cost));

    for connection in sorted_connections {
        let root_start = mf_set.find(coordinates.iter().position(|&c| c == connection.start).unwrap());
        let root_end = mf_set.find(coordinates.iter().position(|&c| c == connection.end).unwrap());

        if root_start != root_end {
            mf_set.union(root_start, root_end);
            result.push(connection);
        }
    }

    result
}



pub fn por(coordinates: Vec<(usize, usize)>, world: &World) -> Vec<Connection> {
    // Sample coordinates and costs
    let connections = connections_list(coordinates.clone(), world);

    // Sample costs for each coordinate to its three closest coordinates
    // You need to replace this with your actual data


    let minimum_spanning_tree = kruskal(coordinates.clone(), connections);




    minimum_spanning_tree
}

fn connections_list(coordinates: Vec<(usize, usize)>, world: &World) -> Vec<Connection> {
    let mut count_connections = vec![0; coordinates.len()];
    let mut connections: Vec<Connection> = Vec::new();

    for (at, coordinate) in coordinates.iter().enumerate() {
        if at == coordinates.len() - 1 {
            break;
        }

        let to_connect = find_closest(&coordinates, at);
        let mut i = 0;
        if count_connections[at] < 4 {
            while count_connections[at] < 4 && i < to_connect.len(){
                let new_connection = Connection {
                    cost: dist(coordinates[at], coordinates[to_connect[i]]),
                    start: coordinate.clone(),
                    end: coordinates[to_connect[i]],
                };
                connections.push(new_connection);
                count_connections[at] += 1;
                count_connections[to_connect[i]] += 1;
                i += 1;
            }
        } else if to_connect.len() > 0{
            let new_connection = Connection {
                cost: dist(coordinates[at], coordinates[to_connect[i]]),
                start: coordinate.clone(),
                end: coordinates[to_connect[i]],
            };
            connections.push(new_connection);
            count_connections[at] += 1;
            count_connections[to_connect[i]] += 1;
        }

    }

    connections
}

fn find_closest(coordinates: &Vec<(usize, usize)>, at: usize) -> Vec<usize> {
    let start = coordinates[at];
    let mut priority = PriorityQueue::new();

    for i in at+1..coordinates.len() {
        let dis = dist(start, coordinates[i].clone());
        priority.push(i, Reverse(dis));
    }

    priority.into_sorted_vec()
}

fn dist(t1: (usize, usize), t2: (usize, usize)) -> usize {
    t1.0.abs_diff(t2.0) + t1.1.abs_diff(t2.1)
}