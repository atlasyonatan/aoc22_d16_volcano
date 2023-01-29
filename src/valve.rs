use petgraph::{
    stable_graph::{IndexType, NodeIndex},
    Direction::Outgoing,
    EdgeType, Graph, Undirected,
};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Valve<Rate, Time> {
    pub name: String,
    pub flow_rate: Rate,
    pub turn_time: Time,
}

pub fn create_valve_graph<Rate: Copy, Time: Copy, TunnelLength: Copy>(
    inputs: Vec<(String, Rate, Vec<String>)>,
    tunnel_length: TunnelLength,
    valve_operation_time: Time,
) -> (
    Graph<Valve<Rate, Time>, TunnelLength, Undirected>,
    HashMap<String, petgraph::stable_graph::NodeIndex>,
) {
    let mut graph = Graph::new_undirected();
    let mut nodes = HashMap::new();
    for (name, flow_rate, _) in inputs.iter() {
        let node = graph.add_node(Valve {
            name: name.to_string(),
            flow_rate: *flow_rate,
            turn_time: valve_operation_time,
        });
        nodes.insert(name.to_string(), node);
    }
    for (name, _, connections) in inputs {
        let node = nodes.get(&name).unwrap();
        for neighbor in connections {
            let neighbor = nodes.get(&neighbor).unwrap();
            graph.add_edge(*node, *neighbor, tunnel_length);
        }
    }
    (graph, nodes)
}

pub fn visit_max_pressures<Ty: EdgeType, Ix: IndexType>(
    graph: &Graph<Valve<u32, u32>, u32, Ty, Ix>,
    visited: &mut HashSet<NodeIndex<Ix>>,
    current: NodeIndex<Ix>,
    time: u32,
) -> u32 {
    let mut pressure = 0;
    for neighbor in graph
        .neighbors_directed(current, Outgoing)
        .filter(|neighbor| !visited.contains(neighbor))
        .collect::<Vec<_>>()
    {
        let edge = graph.find_edge(current, neighbor).unwrap();
        let tunnel_time = graph[edge];
        let valve = &graph[neighbor];
        match time.checked_sub(tunnel_time + valve.turn_time) {
            Some(time_left) => {
                let added_pressure = time_left * valve.flow_rate;

                visited.insert(neighbor);
                let visit_pressure = visit_max_pressures(graph, visited, neighbor, time_left);
                visited.remove(&neighbor);

                pressure = pressure.max(added_pressure + visit_pressure);
            }
            None => continue,
        }
    }
    return pressure;
}

pub fn visit_max_pressures_2<Ty: EdgeType, Ix: IndexType>(
    graph: &Graph<Valve<u32, u32>, u32, Ty, Ix>,
    visited: &mut HashSet<NodeIndex<Ix>>,
    current1: NodeIndex<Ix>,
    time1: u32,
    current2: NodeIndex<Ix>,
    time2: u32,
) -> u32 {
    let mut pressure = 0;
    for neighbor1 in graph
        .neighbors_directed(current1, Outgoing)
        .filter(|neighbor| !visited.contains(neighbor))
        .collect::<Vec<_>>()
    {
        let edge = graph.find_edge(current1, neighbor1).unwrap();
        let tunnel_time = graph[edge];
        let valve = &graph[neighbor1];
        match time1.checked_sub(tunnel_time + valve.turn_time) {
            Some(time_left1) => {
                let added_pressure1 = time_left1 * valve.flow_rate;

                visited.insert(neighbor1);

                for neighbor2 in graph
                    .neighbors_directed(current2, Outgoing)
                    .filter(|neighbor| !visited.contains(neighbor))
                    .collect::<Vec<_>>()
                {
                    let edge = graph.find_edge(current2, neighbor2).unwrap();
                    let tunnel_time = graph[edge];
                    let valve = &graph[neighbor2];
                    match time2.checked_sub(tunnel_time + valve.turn_time) {
                        Some(time_left2) => {
                            let added_pressure2 = time_left2 * valve.flow_rate;

                            visited.insert(neighbor2);

                            let visit_pressure = visit_max_pressures_2(
                                graph, visited, neighbor1, time_left1, neighbor2, time_left2,
                            );
                            visited.remove(&neighbor2);

                            pressure =
                                pressure.max(added_pressure1 + added_pressure2 + visit_pressure);
                        }
                        None => continue,
                    }
                }
                visited.remove(&neighbor1);
            }
            None => continue,
        }
    }
    return pressure;
}

pub struct Walker<Time, Location> {
    pub time: Time,
    pub location: Location,
}

// pub fn multi_visit_max_pressures<Ty: EdgeType, Ix: IndexType>(
//     graph: &Graph<Valve<u32, u32>, u32, Ty, Ix>,
//     walkers: &mut [Walker<u32, NodeIndex<Ix>>],
//     visited: &mut HashSet<NodeIndex<Ix>>,
// ) -> u32 {
//     let mut pressure = 0;
//     let mut stack = Vec::new();
//     for index in 0..walkers.len() {
//         let walker = &mut walkers[index];
//         let current_node = walker.location;
//         let time = walker.time;
//         for neighbor in graph
//             .neighbors_directed(current_node, Outgoing)
//             .filter(|neighbor| !visited.contains(neighbor))
//             .collect::<Vec<_>>()
//         {
//             let edge = graph.find_edge(current_node, neighbor).unwrap();
//             let tunnel_time = graph[edge];
//             let valve = &graph[neighbor];
//             match time.checked_sub(tunnel_time + valve.turn_time) {
//                 Some(time_left) => {
//                     let added_pressure = time_left * valve.flow_rate;

//                     visited.insert(neighbor);
//                     stack.push(neighbor);
//                     // let visit_pressure = visit_max_pressures(graph, neighbor, visited, time_left);
//                     // visited.remove(&neighbor);

//                     // pressure = pressure.max(added_pressure + visit_pressure);
//                 }
//                 None => continue,
//             }
//         }
//     }
//     return pressure;
// }
