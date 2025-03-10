use crate::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::BufReader;

use elevation::GeoTiffElevation;
use fs_err::File;
use geo::{HaversineLength, LineString};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;

pub fn calculate(
    edges: &Vec<Edge>,
    tif_path: &str,
    settings: &Settings,
) -> HashMap<usize, (usize, usize)> {
    println!("Calculating traversal times");
    let progress = ProgressBar::new(edges.len() as u64).with_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})").unwrap());

    let traversal_times: HashMap<usize, (usize, usize)> = edges
        .into_par_iter()
        .progress_with(progress)
        .map(|edge| {
            thread_local!(static ELEVATION: RefCell<Option<GeoTiffElevation<BufReader<File>>>> = RefCell::new(None));
            ELEVATION.with(|elevation_cell| {
                let mut elevation = elevation_cell.borrow_mut();
                if elevation.is_none() {
                    *elevation = Some(GeoTiffElevation::new(BufReader::new(File::open(tif_path).unwrap())));
                }

                let traversal_time = calculate_edge_traversal_time(
                    &edge.linestring,
                    elevation.as_mut().unwrap(),
                    settings.speed,
                    settings.ascention_speed
                );
                (edge.id, traversal_time)
            })
        })
        .collect();
    traversal_times
}

fn calculate_edge_traversal_time(
    linestring: &LineString,
    elevation: &mut GeoTiffElevation<BufReader<File>>,
    speed: f32,
    ascention_speed: f32, // 6 s/m for walking to follow Naismith's rule
) -> (usize, usize) {
    let mut forward_traversal_time: f32 = 0.0;
    let mut backward_traversal_time: f32 = 0.0;

    for line in linestring.lines() {
        let pt1 = line.start;

        let length = line.haversine_length() as f32;

        let height1 = match elevation.get_height_for_lon_lat(pt1.x as f32, pt1.y as f32) {
            Some(height) => height,
            None => {
                // if coordinates outside the UK elevation model, assume flat terrain
                // TODO: remove these coordinates from the graph
                println!("Failed to get height for lon: {}, lat: {}", pt1.x, pt1.y);
                forward_traversal_time += length / speed;
                backward_traversal_time += length / speed;
                continue;
            }
        };
        let pt2 = line.end;
        let height2 = match elevation.get_height_for_lon_lat(pt2.x as f32, pt2.y as f32) {
            Some(height) => height,
            None => {
                println!("Failed to get height for lon: {}, lat: {}", pt2.x, pt2.y);
                forward_traversal_time += length / speed;
                backward_traversal_time += length / speed;
                continue;
            }
        };
        let height_diff = height2 - height1;
        forward_traversal_time += length / speed
            + if height_diff > 0.0 {
                height_diff * ascention_speed
            } else {
                0.0
            };
        backward_traversal_time += length / speed
            + if height_diff < 0.0 {
                -height_diff * ascention_speed
            } else {
                0.0
            };
    }
    let forward: usize = if forward_traversal_time <= 1.0 {
        1
    } else {
        forward_traversal_time.round() as usize
    };
    let backward: usize = if backward_traversal_time <= 1.0 {
        1
    } else {
        backward_traversal_time.round() as usize
    };
    (forward, backward)
}
