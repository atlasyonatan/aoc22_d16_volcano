use std::ops::Add;

use petgraph::{
    stable_graph::{IndexType, NodeIndex},
    EdgeType, Graph,
};

pub fn connect_neighbors_min<N, E: Copy + Add<Output = E> + Ord, Ty: EdgeType, Ix: IndexType>(
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
