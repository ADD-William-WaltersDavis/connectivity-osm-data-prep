use std::cell::RefCell;
use std::io::BufReader;
use crate::*;

use fs_err::File;
use elevation::GeoTiffElevation;
use geo::{LineString, HaversineLength};
use indicatif::{ProgressBar, ProgressStyle, ParallelProgressIterator};
use rayon::prelude::*;

pub fn process(edges: &Vec<Edge>, tif_path: &str) -> Vec<(usize, (u16, u16))> {

    println!("Calculating traversal times");
    let progress = ProgressBar::new(edges.len() as u64).with_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})").unwrap());

    let walking_speed: f32 = 1.33; // m/s
    // Naismith's rule: 1 hour for every 600 m of ascent
    let naismith_constant: f32 = 6.0; // s/m of vertical ascent

    let traversal_times: Vec<(usize, (u16, u16))> = edges
        .into_par_iter()
        .progress_with(progress)
        .map(|edge| {
            thread_local!(static ELEVATION: RefCell<Option<GeoTiffElevation<BufReader<File>>>> = RefCell::new(None));
            ELEVATION.with(|elevation_cell| {
                let mut elevation = elevation_cell.borrow_mut();
                if elevation.is_none() {
                    *elevation = Some(GeoTiffElevation::new(BufReader::new(File::open(tif_path).unwrap())));
                }

                let traversal_time = calculate_edge_traversal_time(&edge.linestring, elevation.as_mut().unwrap(), walking_speed, naismith_constant);
                (edge.id, traversal_time)
            })
           
        })
        .collect();
    traversal_times
}

fn calculate_edge_traversal_time(
    linestring: &LineString, 
    elevation: &mut GeoTiffElevation<BufReader<File>>, 
    walking_speed: f32, 
    naismith_constant: f32
) -> (u16, u16) {
    let mut forward_traversal_time: f32 = 0.0;
    let mut backward_traversal_time: f32 = 0.0;

    for line in linestring.lines() {
        let pt1 = line.start;
        let height1 = elevation
            .get_height_for_lon_lat(pt1.x as f32, pt1.y as f32)
            .unwrap();
        let pt2 = line.end;
        let height2 = elevation
            .get_height_for_lon_lat(pt2.x as f32, pt2.y as f32)
            .unwrap();
        let length = line.haversine_length() as f32;
        let height_diff = height2 - height1;

        forward_traversal_time += length / walking_speed + if height_diff > 0.0 { height_diff * naismith_constant } else { 0.0 };
        backward_traversal_time += length / walking_speed + if height_diff < 0.0 { height_diff * naismith_constant } else { 0.0 };
    }
    let forward: u16 = if forward_traversal_time <= 1.0 { 1 } else { forward_traversal_time.round() as u16 };
    let backward: u16 = if backward_traversal_time <= 1.0 { 1 } else { backward_traversal_time.round() as u16 };
    (forward, backward)
}