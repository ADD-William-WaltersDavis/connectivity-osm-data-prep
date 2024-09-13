use graph_from_pbf::Edge;

use geo::{LineString, Point, RhumbBearing};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct Angles {
    pub forward_arrival: u16,
    pub forward_departure: u16,
    pub backward_arrival: u16,
    pub backward_departure: u16,
}

pub fn calculate(edges: &Vec<Edge>) -> HashMap<usize, Angles> {
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

fn arrival_departure_angle_from_north(linestring: &LineString) -> Angles {
    let first_point = linestring.points().next().unwrap();
    let second_point = linestring.points().nth(1).unwrap();
    let last_point = linestring.points().last().unwrap();
    let second_last_point = linestring.points().nth_back(1).unwrap();

    Angles {
        forward_arrival: get_angle(&first_point, &second_point),
        forward_departure: get_angle(&second_last_point, &last_point),
        backward_arrival: get_angle(&last_point, &second_last_point),
        backward_departure: get_angle(&second_point, &first_point),
    }
}

fn get_angle(a: &Point, b: &Point) -> u16 {
    let angle_from_north = a.rhumb_bearing(*b).round() as u16;
    angle_from_north
}
