use std::collections::HashMap;

use anyhow::Result;
use destinations_from_pbf::{read_nodes, Destination};

use kdtree::{distance::squared_euclidean, KdTree};

pub fn process(destinations: &Vec<Destination>, mode: &str) -> Result<Vec<[u16; 3]>> {
    let nodes = read_nodes(mode)?;
    let mut graph_values: Vec<[u16; 3]> = vec![[0; 3]; nodes.len()];

    let dimensions = 2;
    let mut kdtree: KdTree<f64, usize, [f64; 2]> = KdTree::new(dimensions);

    for (node_id, coord) in nodes.iter() {
        kdtree.add([coord.x, coord.y], *node_id)?;
    }

    let osm_amenity_to_subpurpose = {
        let mut map = HashMap::new();
        map.insert("hospital".to_string(), 0);
        map.insert("doctors".to_string(), 1);
        map.insert("clinic".to_string(), 1);
        map.insert("pharmacy".to_string(), 2);
        map
    };

    // let nearest = kdtree.nearest(&[0.745147, 51.3577695], 1, &squared_euclidean)?;

    for desination in destinations.iter() {
        let nearest = kdtree.nearest(
            &[desination.centroid.x(), desination.centroid.y()],
            1,
            &squared_euclidean,
        )?;
        let node_id = nearest[0].1;
        let subpurpose_index = osm_amenity_to_subpurpose
            .get(&desination.subpurpose)
            .unwrap();
        graph_values[*node_id][*subpurpose_index] += 1;
    }
    Ok(graph_values)
}
