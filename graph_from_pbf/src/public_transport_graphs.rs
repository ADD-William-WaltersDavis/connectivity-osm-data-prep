use crate::pt_stops::add_stops;
use anyhow::Result;
use geo::Coord;
use graph_from_pbf::{read_pt_stops, read_timetables, Timetable};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct NodeWalk {
    pub has_pt: bool,
    pub edges: Vec<(usize, usize)>,
}

#[derive(Serialize)]
pub struct NodeRoute {
    pub next_stop_node: usize,
    pub timetable: Timetable,
}

pub fn process(
    walk_graph: Vec<Vec<(usize, usize, u16, u16, u32)>>,
    walk_nodes: HashMap<usize, Coord>,
) -> Result<(Vec<NodeWalk>, Vec<NodeRoute>)> {
    let walk_graph_length = walk_graph.len();
    // for pt graph walk we ignore turning angles and linkIDs
    let mut pt_graph_walk: Vec<NodeWalk> = walk_graph
        .into_iter()
        .map(|link| {
            let mut edges = Vec::new();
            for (traversal_time, to, _, _, _) in link {
                edges.push((traversal_time, to));
            }
            NodeWalk {
                has_pt: false,
                edges,
            }
        })
        .collect();

    let pt_stops = read_pt_stops()?;
    add_stops(
        &pt_stops,
        walk_nodes,
        &mut pt_graph_walk,
        &walk_graph_length,
    );

    let timetables = read_timetables()?;

    // create pt graph routes and pad with empty NodeRoutes for non-pt route nodes
    let mut pt_graph_routes: Vec<NodeRoute> = Vec::new();
    for _ in 0..pt_graph_walk.len() {
        pt_graph_routes.push(NodeRoute {
            next_stop_node: 0,
            timetable: Timetable(Vec::new()),
        });
    }

    let length_before_routes = pt_graph_walk.len();
    for (i, input_timetable) in timetables.iter().enumerate() {
        let pt_stop_node = walk_graph_length + input_timetable.pt_stop_node;
        // add pt route nodes to pt_graph_walk
        // check if pt stop has a next node else set has_pt to false
        let mut has_pt = true;
        if input_timetable.next_node.is_none() {
            has_pt = false;
        }
        pt_graph_walk.push(NodeWalk {
            has_pt,
            edges: vec![(0, pt_stop_node)],
        });
        // add pt route node edge to pt stop node
        pt_graph_walk[pt_stop_node]
            .edges
            .push((0, length_before_routes + i));
        // add pt
        pt_graph_routes.push(NodeRoute {
            next_stop_node: length_before_routes + input_timetable.next_node.unwrap_or(0),
            timetable: input_timetable
                .timetable
                .clone()
                .unwrap_or(Timetable(Vec::new())),
        })
    }
    Ok((pt_graph_walk, pt_graph_routes))
}
