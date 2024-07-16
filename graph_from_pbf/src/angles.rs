use graph_from_pbf::{Angles, Edge};

use std::collections::HashMap;
use geo::{LineString, RhumbBearing};
use indicatif::{ProgressBar, ProgressStyle, ParallelProgressIterator};
use rayon::prelude::*;

pub fn process(edges: &Vec<Edge>) -> HashMap<usize, Angles> {

    println!("Calculating angle from north of arrival and departure");
    let progress = ProgressBar::new(edges.len() as u64).with_style(ProgressStyle::with_template(
        "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({per_sec}, {eta})").unwrap());

    let angles: HashMap<usize, Angles> = edges
        .into_par_iter()
        .progress_with(progress)
        .map(|edge| {
            let angles: Angles = arrival_departure_angle_from_north(&edge.linestring);
            (edge.id, angles)
        })
        .collect();
    angles
}

fn arrival_departure_angle_from_north (
    linestring: &LineString,
) -> Angles {
    let first_point = linestring.points().next().unwrap();
    let second_point = linestring.points().nth(1).unwrap();
    let last_point = linestring.points().last().unwrap();
    let second_last_point = linestring.points().nth_back(1).unwrap();

    Angles {
        forward_arrival: first_point.rhumb_bearing(second_point).round() as u16,
        forward_departure: second_last_point.rhumb_bearing(last_point).round() as u16,
        backward_arrival: last_point.rhumb_bearing(second_last_point).round() as u16,
        backward_departure: second_point.rhumb_bearing(first_point).round() as u16,
    }
}