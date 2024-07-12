use crate::*;

use geo::{LineString, RhumbBearing};
use indicatif::{ProgressBar, ProgressStyle, ParallelProgressIterator};
use rayon::prelude::*;

pub fn process(edges: &Vec<Edge>) -> Vec<(usize, (i32, i32))> {

    println!("Calculating angle from north of arrival and departure");
    let progress = ProgressBar::new(edges.len() as u64).with_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})").unwrap());

    let angles: Vec<(usize, (i32, i32))> = edges
        .into_par_iter()
        .progress_with(progress)
        .map(|edge| {
            let (arrival_angle, departure_angle): (i32, i32) = arrival_departure_angle_from_north(&edge.linestring);
            (edge.id, (arrival_angle, departure_angle))
        })
        .collect();
    angles
}

fn arrival_departure_angle_from_north (
    linestring: &LineString,
) -> (i32, i32) {
   // select first two points in linestring
    let first_point = linestring.points().next().unwrap();
    let second_point = linestring.points().nth(1).unwrap();
    // calculate angle from north
    let angle_arrival = first_point.rhumb_bearing(second_point).round() as i32;

    // select last two points in linestring
    let last_point = linestring.points().last().unwrap();
    let second_last_point = linestring.points().nth_back(1).unwrap();
    // calculate angle from north
    let angle_departure = second_last_point.rhumb_bearing(last_point).round() as i32;
    (angle_arrival, angle_departure)
}