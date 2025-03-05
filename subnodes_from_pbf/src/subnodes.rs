use crate::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::BufReader;

use elevation::GeoTiffElevation;
use fs_err::File;
use geo::{Coord, HaversineLength, Line, LineString};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;

pub fn process(
    edges: &Vec<Edge>,
    tif_path: &str,
    settings: &Settings,
    graph_node_lookup: HashMap<i64, (usize, Coord)>,
) -> Vec<SubNode> {
    println!("Getting subnodes");
    let progress = ProgressBar::new(edges.len() as u64).with_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})").unwrap());

    let subnodes: Vec<SubNode> = edges
        .into_par_iter()
        .progress_with(progress)
        .flat_map(|edge| {
            thread_local!(static ELEVATION: RefCell<Option<GeoTiffElevation<BufReader<File>>>> = RefCell::new(None));
            ELEVATION.with(|elevation_cell| {
                let mut elevation = elevation_cell.borrow_mut();
                if elevation.is_none() {
                    *elevation = Some(GeoTiffElevation::new(BufReader::new(File::open(tif_path).unwrap())));
                }
                calculate_subnodes(
                    &edge.linestring,
                    &edge.start_node,
                    &edge.end_node,
                    elevation.as_mut().unwrap(),
                    settings.speed,
                    settings.ascention_speed,
                    &graph_node_lookup,
                )
            })
        })
        .collect();
    subnodes
}

struct ComponentLine {
    line: Line,
    length: f32,
    forward_traversal_time: f32,
    backward_traversal_time: f32,
}

fn get_traversal_times(
    linestring: &LineString,
    elevation: &mut GeoTiffElevation<BufReader<File>>,
    speed: f32,
    ascention_speed: f32, // 6 s/m for walking to follow Naismith's rule
) -> (f32, f32, Vec<ComponentLine>) {
    let mut link_forward_traversal_time: f32 = 0.0;
    let mut link_backward_traversal_time: f32 = 0.0;
    let mut component_lines: Vec<ComponentLine> = Vec::new();

    for line in linestring.lines() {
        let pt1 = line.start;

        let length = line.haversine_length() as f32;

        let height1 = match elevation.get_height_for_lon_lat(pt1.x as f32, pt1.y as f32) {
            Some(height) => height,
            None => {
                // catch if coordinates outside the UK elevation model, assume flat terrain
                // TODO: remove these coordinates from the graph
                println!("Failed to get height for lon: {}, lat: {}", pt1.x, pt1.y);
                link_forward_traversal_time += length / speed;
                link_backward_traversal_time += length / speed;
                component_lines.push(ComponentLine {
                    line: line,
                    length: length,
                    forward_traversal_time: length / speed,
                    backward_traversal_time: length / speed,
                });
                continue;
            }
        };
        let pt2 = line.end;
        let height2 = match elevation.get_height_for_lon_lat(pt2.x as f32, pt2.y as f32) {
            Some(height) => height,
            None => {
                println!("Failed to get height for lon: {}, lat: {}", pt2.x, pt2.y);
                link_forward_traversal_time += length / speed;
                link_backward_traversal_time += length / speed;
                component_lines.push(ComponentLine {
                    line: line,
                    length: length,
                    forward_traversal_time: length / speed,
                    backward_traversal_time: length / speed,
                });
                continue;
            }
        };
        let height_diff = height2 - height1;
        let forward_traversal_time = length / speed
            + if height_diff > 0.0 {
                height_diff * ascention_speed
            } else {
                0.0
            };
        let backward_traversal_time = length / speed
            + if height_diff < 0.0 {
                -height_diff * ascention_speed
            } else {
                0.0
            };
        link_forward_traversal_time += forward_traversal_time;
        link_backward_traversal_time += backward_traversal_time;
        component_lines.push(ComponentLine {
            line: line,
            length: length,
            forward_traversal_time: forward_traversal_time,
            backward_traversal_time: backward_traversal_time,
        });
    }
    (
        link_forward_traversal_time,
        link_backward_traversal_time,
        component_lines,
    )
}

fn get_subnode_coords(fraction_across_line: f64, line: &Line) -> (f64, f64) {
    (
        (1.0 - fraction_across_line) * line.start.x + fraction_across_line * line.end.x,
        (1.0 - fraction_across_line) * line.start.y + fraction_across_line * line.end.y,
    )
}
fn get_subnodes(
    start_node_id: usize,
    end_node_id: usize,
    link_forward_traversal_time: f32,
    _link_backward_traversal_time: f32, // don't use could remove
    component_lines: Vec<ComponentLine>,
) -> Vec<SubNode> {
    let mut subnodes: Vec<SubNode> = Vec::new();

    // keep track how far along the link we are
    let mut time_to_component_line_start_forward = 0.0;
    let mut time_to_component_line_start_backward = 0.0;

    // add first subnode of the link
    subnodes.push(SubNode {
        start_node: start_node_id,
        end_node: end_node_id,
        longitude: component_lines[0].line.start.x,
        latitude: component_lines[0].line.start.y,
        time_to_start: 0,
        time_to_end: link_forward_traversal_time.round() as usize,
    });
    for component_line in component_lines.iter() {
        // TODO: adjust this threshold based on the observed accuracy
        // currently max distance between 7.5m and minumum ~7.5/2 = 3.75m
        if component_line.length > 7.5 {
            let n_inner_subnodes = (component_line.length / 5.0).floor() as usize;

            for inner_subnode_index in 1..n_inner_subnodes + 1 {
                let fraction_across_line =
                    inner_subnode_index as f32 / (n_inner_subnodes + 1) as f32;

                let forward_time_from_subnode =
                    fraction_across_line * component_line.forward_traversal_time;
                let backward_time_from_subnode =
                    fraction_across_line * component_line.backward_traversal_time;
                let (subnode_x, subnode_y) =
                    get_subnode_coords(fraction_across_line as f64, &component_line.line);
                subnodes.push(SubNode {
                    start_node: start_node_id,
                    end_node: end_node_id,
                    longitude: subnode_x,
                    latitude: subnode_y,
                    time_to_start: (time_to_component_line_start_backward
                        + backward_time_from_subnode)
                        .round() as usize,
                    time_to_end: (link_forward_traversal_time
                        - (time_to_component_line_start_forward + forward_time_from_subnode))
                        .round() as usize,
                });
            }
        }
        // add the last subnode of component line
        subnodes.push(SubNode {
            start_node: start_node_id,
            end_node: end_node_id,
            longitude: component_line.line.end.x,
            latitude: component_line.line.end.y,
            time_to_start: (time_to_component_line_start_backward
                + component_line.backward_traversal_time)
                .round() as usize,
            time_to_end: (link_forward_traversal_time
                - (time_to_component_line_start_forward + component_line.forward_traversal_time))
                .round() as usize,
        });
        time_to_component_line_start_forward += component_line.forward_traversal_time;
        time_to_component_line_start_backward += component_line.backward_traversal_time;
    }
    subnodes
}

fn calculate_subnodes(
    linestring: &LineString,
    start_node: &i64,
    end_node: &i64,
    elevation: &mut GeoTiffElevation<BufReader<File>>,
    speed: f32,
    ascention_speed: f32,
    graph_node_lookup: &HashMap<i64, (usize, Coord)>,
) -> Vec<SubNode> {
    let (link_forward_traversal_time, link_backward_traversal_time, line_details) =
        get_traversal_times(linestring, elevation, speed, ascention_speed);

    let start_node_id = graph_node_lookup[start_node].0;
    let end_node_id = graph_node_lookup[end_node].0;

    get_subnodes(
        start_node_id,
        end_node_id,
        link_forward_traversal_time,
        link_backward_traversal_time,
        line_details,
    )
}
