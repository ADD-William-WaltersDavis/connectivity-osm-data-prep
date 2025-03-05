use crate::*;
use std::collections::{HashMap, HashSet};

use geo::{Coord, LineString};
use indicatif::{ProgressBar, ProgressStyle};
use osm_reader::{Element, NodeID, WayID};

pub fn process(
    osm_paths: Vec<&str>,
    settings: &Settings,
) -> Result<(HashMap<i64, (usize, Coord)>, Vec<Edge>)> {
    let (node_mapping, ways) = scrape_osm(osm_paths, &settings)?;
    let edges: Vec<Edge> = split_ways_into_edges(&node_mapping, ways);
    let graph_nodes_lookup = get_graph_nodes_lookup(node_mapping, &edges);
    Ok((graph_nodes_lookup, edges))
}

fn scrape_osm(
    osm_paths: Vec<&str>,
    settings: &Settings,
) -> Result<(
    HashMap<NodeID, Coord>,
    Vec<(WayID, Vec<NodeID>, bool, bool)>,
)> {
    let mut node_mapping: HashMap<NodeID, Coord> = HashMap::new();
    let mut highways: Vec<(WayID, Vec<NodeID>, bool, bool)> = Vec::new();
    let mut unique_ways: HashSet<WayID> = HashSet::new();
    for osm_path in osm_paths {
        let mut first_way = true;
        println!("Reading {osm_path}");
        let nodes_progress = ProgressBar::new_spinner().with_style(
            ProgressStyle::with_template("[{elapsed_precise}] {human_len} nodes read ({per_sec})")
                .unwrap(),
        );
        let ways_progress = ProgressBar::new_spinner().with_style(
            ProgressStyle::with_template("[{elapsed_precise}] {human_len} ways read ({per_sec})")
                .unwrap(),
        );
        osm_reader::parse(&fs_err::read(osm_path)?, |elem| match elem {
            Element::Node { id, lon, lat, .. } => {
                nodes_progress.inc(1);
                node_mapping.insert(id, Coord { x: lon, y: lat });
            }
            Element::Way { id, node_ids, tags } => {
                if tags.contains_key("highway")
                    // select just ways meeting mode criteria
                    && settings.tag_pairs.iter().all(|(k, v)| tags.get(k) != Some(v))
                    && unique_ways.get(&id).is_none()
                {
                    // TODO: add oneway tag filtering here
                    if first_way {
                        nodes_progress.finish();
                        first_way = false;
                    }
                    ways_progress.inc(1);
                    let (forward, backward) = oneway_access(&tags, &settings);
                    highways.push((id, node_ids, forward, backward));
                    unique_ways.insert(id);
                }
            }
            Element::Relation { .. } | Element::Bounds { .. } => {}
        })?;
        ways_progress.finish();
    }

    Ok((node_mapping, highways))
}

fn split_ways_into_edges(
    node_mapping: &HashMap<NodeID, Coord>,
    ways: Vec<(WayID, Vec<NodeID>, bool, bool)>,
) -> Vec<Edge> {
    println!("Splitting ways into edges");

    // Count how many ways reference each node
    let mut node_counter: HashMap<NodeID, usize> = HashMap::new();
    for (_, node_ids, _, _) in &ways {
        for node in node_ids {
            *node_counter.entry(*node).or_insert(0) += 1;
        }
    }

    // Split each way into edges
    let progress = ProgressBar::new(ways.len() as u64).with_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})").unwrap());
    let mut edges: Vec<Edge> = Vec::new();
    let mut edge_id: usize = 0;
    for (way_id, node_ids, forward, backward) in ways {
        progress.inc(1);
        let mut pts = Vec::new();
        let mut start_node = node_ids[0].clone();

        let num_nodes = node_ids.len();
        for (idx, node) in node_ids.into_iter().enumerate() {
            pts.push(node_mapping[&node]);
            // Edges start/end at intersections between two ways. The endpoints of the way also
            // count as intersections.
            let is_endpoint =
                idx == 0 || idx == num_nodes - 1 || *node_counter.get(&node).unwrap() > 1;
            if is_endpoint && pts.len() > 1 {
                edges.push(Edge {
                    id: edge_id,
                    osm_id: way_id.0,
                    start_node: start_node.0,
                    end_node: node.0,
                    linestring: LineString::new(std::mem::take(&mut pts)),
                    forward: forward,
                    backward: backward,
                });
                edge_id += 1;
                start_node = node;
                // Start the next edge
                pts.push(node_mapping[&node]);
            }
        }
    }
    progress.finish();
    edges
}

fn get_graph_nodes_lookup(
    node_mapping: HashMap<NodeID, Coord>,
    edges: &Vec<Edge>,
) -> HashMap<i64, (usize, Coord)> {
    let mut graph_nodes_lookup: HashMap<i64, (usize, Coord)> = HashMap::new();
    let mut graph_node_id: usize = 0;
    for edge in edges {
        if !graph_nodes_lookup.contains_key(&edge.start_node) {
            graph_nodes_lookup.insert(
                edge.start_node,
                (graph_node_id, node_mapping[&NodeID(edge.start_node)]),
            );
            graph_node_id += 1;
        }
        if !graph_nodes_lookup.contains_key(&edge.end_node) {
            graph_nodes_lookup.insert(
                edge.end_node,
                (graph_node_id, node_mapping[&NodeID(edge.end_node)]),
            );
            graph_node_id += 1;
        }
    }
    graph_nodes_lookup
}

fn oneway_access(tags: &HashMap<String, String>, settings: &Settings) -> (bool, bool) {
    let forward = true;
    let mut backward = true;
    if settings.mode != "walk" {
        if tags.get("oneway") == Some(&"yes".to_string()) {
            backward = false;
        }
        if settings.mode == "cycling" {
            if tags.get("oneway:bicycle") == Some(&"no".to_string()) {
                backward = true;
            }
        }
    }
    (forward, backward)
}
