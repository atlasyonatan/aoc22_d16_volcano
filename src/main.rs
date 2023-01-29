use petgraph::{algo::dijkstra, dot::Dot, visit::IntoNodeReferences};
use regex::Regex;
use std::collections::HashSet;

use std::{
    fs::File,
    io::{self, BufRead},
    path::Path,
};
pub mod valve;
use crate::graph_tools::connect_neighbors_min;
use crate::valve::{create_valve_graph, visit_max_pressures};

fn main() {
    let file_path = "../test.txt";
    let time = 30;
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

    // modify the graph such that every node has an edge to every other node with weight equal to the shortest path between the nodes in the graph;
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

    let start = *nodes.get(start_node).unwrap();

    let mut visited = HashSet::new();
    visited.insert(start);
    let max_pressure = visit_max_pressures(&graph, start, &mut visited, time);
    println!("{}", max_pressure);
}

pub mod graph_tools;
