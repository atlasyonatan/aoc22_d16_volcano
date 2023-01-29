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
    current: NodeIndex<Ix>,
    visited: &mut HashSet<NodeIndex<Ix>>,
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
                let visit_pressure = visit_max_pressures(graph, neighbor, visited, time_left);
                visited.remove(&neighbor);

                pressure = pressure.max(added_pressure + visit_pressure);
            }
            None => continue,
        }
    }
    return pressure;
}
