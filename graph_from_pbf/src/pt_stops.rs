use crate::public_transport_graphs::NodeWalk;
use geo::{Coord, HaversineDistance, Point};
use kdtree::distance::squared_euclidean;
use kdtree::KdTree;
use std::collections::HashMap;

pub fn add_stops(
    pt_stops: &Vec<(usize, Coord)>,
    walk_nodes: HashMap<usize, Coord>,
    pt_graph_walk: &mut Vec<NodeWalk>,
    walk_graph_length: &usize,
) {
    let kdtree = create_tree(&walk_nodes);

    // pad pt_graph_walk with empty NodeWalks for pt stops
    for _ in 0..pt_stops.len() {
        pt_graph_walk.push(NodeWalk {
            has_pt: false,
            edges: Vec::new(),
        });
    }

    // for each pt stop find the nearest two walk nodes and add an edges
    for (id, coord) in pt_stops {
        let pt_graph_walk_id = *walk_graph_length + id;
        let search_coord: [f64; 2] = [coord.x, coord.y];
        let result = kdtree
            .nearest(&search_coord, 2, &squared_euclidean)
            .unwrap();
        for (_, node_id) in result {
            // TOOD use traversal time with topography or use nearest link
            let traversal_time =
                Point(walk_nodes[&node_id]).haversine_distance(&Point(*coord)) / 1.33;
            pt_graph_walk[pt_graph_walk_id]
                .edges
                .push((traversal_time as usize, *node_id));
            pt_graph_walk[*node_id]
                .edges
                .push((traversal_time as usize, pt_graph_walk_id));
        }
    }
}

fn create_tree(walk_nodes: &HashMap<usize, Coord>) -> KdTree<f64, usize, [f64; 2]> {
    let dimensions = 2;
    let mut kdtree = KdTree::new(dimensions);

    for (node_id, coord) in walk_nodes {
        kdtree.add([coord.x, coord.y], node_id.clone()).unwrap();
    }
    kdtree
}