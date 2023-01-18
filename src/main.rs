use petgraph::{
    algo::{all_simple_paths, dijkstra},
    dot::Dot,
    stable_graph::{IndexType, NodeIndex},
    visit::IntoNodeReferences,
    EdgeType, Graph, Undirected,
};
use regex::Regex;
use std::ops::Add;
use std::{collections::HashMap};
use std::{
    fs::File,
    io::{self, BufRead},
    path::Path,
};

fn main() {
    let file_path = "../test.txt";
    let path = Path::new(file_path);
    let file = File::open(path).unwrap();
    let reg =
        Regex::new(r"^Valve (\w+) has flow rate=(\d+); tunnels? leads? to valves? ([\w, ]+)$")
            .unwrap();
    let inputs: Vec<_> = io::BufReader::new(file)
        .lines()
        .map(|result| result.unwrap())
        .map(|string| {
            let captures = reg
                .captures(&string)
                .unwrap_or_else(|| panic!("Invalid input line: \"{}\"", string));
            let name = captures.get(1).unwrap().as_str().to_string();
            let flow_rate = captures.get(2).unwrap().as_str().parse::<u32>().unwrap();
            let tunnel_connections: Vec<_> = captures
                .get(3)
                .unwrap()
                .as_str()
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            (name, flow_rate, tunnel_connections)
        })
        .collect();

    let (mut graph, nodes) = create_valve_graph(inputs, 1u32, 1u32);
    println!("{:?}", Dot::with_config(&graph, &[]));

    //simplify graph to only relevant nodes
    let mut irrelevant: Vec<_> = nodes
        .iter()
        .filter_map(|(name, &node)| (graph[node].flow_rate == 0).then(|| name.as_str()))
        .collect();
    let start_node = "AA";
    irrelevant.retain(|&name| name != start_node);

    for name in irrelevant {
        let (node, _) = graph
            .node_references()
            .find(|&(_, valve)| valve.name == name)
            .unwrap();
        connect_neighbors_min(&mut graph, node, false);
        graph.remove_node(node);
    }

    println!("{:?}", Dot::with_config(&graph, &[]));

    // modify the graph such that every node only has an edge to every other node with weight equal to the shortest path between the nodes in the graph;
    for node in graph.node_indices() {
        let mut map = dijkstra(&graph, node, None, |edge| *edge.weight());
        map.remove(&node);
        // remove all edges whose source is the current node
        graph.retain_edges(|g, e| g.edge_endpoints(e).unwrap().0 != node);
        for (destination, length) in map {
            graph.add_edge(node, destination, length);
        }
    }

    println!("{:?}", Dot::with_config(&graph, &[]));

    let walk_length = 30;
    let start = nodes.get(start_node).unwrap();
    
    // instead: iterate permutations of paths that visit each node once.
    let paths = all_simple_paths::<Vec<_>, _>(
        &graph,
        *start,
        *start,
        0,
        Some(walk_length + 1),
    );
    let max_pressure = paths
        .map(|path| pressure_for_path(&graph, &path, walk_length as u32))
        .max()
        .unwrap();
    println!("part 1: {}", max_pressure);
   
    // let max = max_pressure(&mut graph, *nodes.get("AA").unwrap(), 0, 30);
    // println!("max pressure: {}", max)
}

fn pressure_for_path<Ty: EdgeType, Ix: IndexType>(
    graph: &Graph<Valve<u32, u32>, u32, Ty, Ix>,
    path: &[NodeIndex<Ix>],
    mut time: u32,
) -> u32 {
    let mut pressure = 0;
    let i = 0;
    while i < path.len() - 1 {
        //move from i to i+1
        let (from, to) = (path[i], path[i + 1]);
        let edge = graph.find_edge(from, to).unwrap();
        let walk_time = graph[edge];
        match time.checked_sub(walk_time) {
            Some(time_left) => time = time_left,
            None => break,
        }

        //open newly visited valve
        let valve = &graph[to];
        match time.checked_sub(valve.turn_time) {
            Some(time_left) => time = time_left,
            None => break,
        }
        pressure += valve.flow_rate * time;
    }
    return pressure;
}

fn connect_neighbors_min<N, E: Copy + Add<Output = E> + Ord, Ty: EdgeType, Ix: IndexType>(
    graph: &mut Graph<N, E, Ty, Ix>,
    node: NodeIndex<Ix>,
    allow_self: bool,
) {
    let predecessors: Vec<_> = graph
        .neighbors_directed(node, petgraph::Direction::Incoming)
        .collect();
    let succcessors: Vec<_> = graph
        .neighbors_directed(node, petgraph::Direction::Outgoing)
        .collect();
    for &i in predecessors.iter() {
        let edge_from_i = graph.find_edge(i, node).unwrap();
        for &j in succcessors.iter() {
            if !allow_self && i == j {
                continue;
            }
            let edge_to_j = graph.find_edge(node, j).unwrap();
            let new_weight = graph[edge_from_i] + graph[edge_to_j];
            if let Some(edge) = graph.find_edge(i, j) {
                graph[edge] = graph[edge].min(new_weight);
            } else {
                graph.add_edge(i, j, new_weight);
            }
        }
    }
}

#[derive(Debug)]
struct Valve<Rate, Time> {
    name: String,
    flow_rate: Rate,
    turn_time: Time,
}

fn create_valve_graph<Rate: Copy, Time: Copy, TunnelLength: Copy>(
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

// fn max_pressure(
//     graph: &mut Graph<Valve<u32, u32>, u32, Undirected>,
//     start: NodeIndex,
//     total_pressure: u32,
//     length: u32,
// ) -> u32 {
//     if length == 0 {
//         return total_pressure;
//     }
//     let mut max = total_pressure;

//     //continue walking without opening valve
//     let mut walker = graph.neighbors(start).detach();
//     while let Some((edge, node)) = walker.next(graph) {
//         if let Some(length) = length.checked_sub(graph[edge]) {
//             max = max.max(max_pressure(graph, node, total_pressure, length));
//         }
//     }
//     if graph[start].flow_rate > 0 {
//         if let Some(length) = length.checked_sub(graph[start].turn_time) {
//             //open valve
//             let total_pressure = total_pressure + graph[start].flow_rate * length;
//             let flow_rate = graph[start].flow_rate;
//             graph[start].flow_rate = 0;
//             max = max.max(total_pressure);
//             let mut walker = graph.neighbors(start).detach();
//             while let Some((edge, node)) = walker.next(graph) {
//                 if let Some(length) = length.checked_sub(graph[edge]) {
//                     max = max.max(max_pressure(graph, node, total_pressure, length));
//                 }
//             }
//             graph[start].flow_rate = flow_rate;
//         }
//     }
//     max
// }
