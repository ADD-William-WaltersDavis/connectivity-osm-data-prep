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

#[derive(Serialize)]
pub struct ReverseNodeRoute {
    pub prev_stop_node: usize,
    pub timetable: Timetable,
}

pub fn process(
    walk_graph: Vec<Vec<(usize, usize, u16, u16, u32)>>,
    walk_nodes: HashMap<usize, Coord>,
) -> Result<(Vec<NodeWalk>, Vec<NodeRoute>, Vec<ReverseNodeRoute>)> {
    println!("Creating public transport graphs");
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

    println!("Creating public transport routes graph");
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
        let mut next_stop_node = length_before_routes + input_timetable.next_node.unwrap_or(0);
        if input_timetable.next_node.is_none() {
            has_pt = false;
            next_stop_node = 0;
        }
        pt_graph_walk.push(NodeWalk {
            has_pt,
            edges: vec![(0, pt_stop_node)],
        });
        // add pt route node edge to pt stop node
        pt_graph_walk[pt_stop_node]
            .edges
            .push((0, length_before_routes + i));
        // add pt route node edge to next node
        pt_graph_routes.push(NodeRoute {
            next_stop_node,
            timetable: input_timetable
                .timetable
                .clone()
                .unwrap_or(Timetable(Vec::new())),
        })
    }
    assert_eq!(pt_graph_walk.len(), pt_graph_routes.len());

    let pt_graph_routes_reverse = reverse_graph_routes(&pt_graph_routes);

    assert_eq!(pt_graph_walk.len(), pt_graph_routes_reverse.len());

    Ok((pt_graph_walk, pt_graph_routes, pt_graph_routes_reverse))
}

fn reverse_graph_routes(pt_graph_routes: &Vec<NodeRoute>) -> Vec<ReverseNodeRoute> {
    let mut pt_graph_routes_reverse: Vec<ReverseNodeRoute> = Vec::new();
    // fill with empty ReverseNodeRoutes
    for _ in 0..pt_graph_routes.len() {
        pt_graph_routes_reverse.push(ReverseNodeRoute {
            prev_stop_node: 0,
            timetable: Timetable(Vec::new()),
        });
    }
    // populate ReverseNodeRoutes with prev_stop_node and timetable
    for (i, node_route) in pt_graph_routes.iter().enumerate() {
        if node_route.next_stop_node != 0 {
            let mut reversed_timetable: Timetable = node_route.timetable.clone();
            reversed_timetable.reverse();
            pt_graph_routes_reverse[node_route.next_stop_node] = ReverseNodeRoute {
                prev_stop_node: i,
                timetable: reversed_timetable,
            };
        }
    }
    pt_graph_routes_reverse
}
