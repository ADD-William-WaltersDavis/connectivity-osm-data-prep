use crate::edges::DrivingEdge;

use geo::{LineString, Point, RhumbBearing, HaversineLength};
use driving_graph::OSNodeID;
use std::collections::HashMap;

pub fn create_driving_graph(
    edges: &Vec<DrivingEdge>,
    osid_to_graph_id: &HashMap<OSNodeID, usize>,
    time_group: &str,
) -> Vec<Vec<(usize, usize, u16, u16, u32)>> {
    println!("Creating driving graph for time group: {}", time_group);
    let mut driving_graph: Vec<Vec<(usize, usize, u16, u16, u32)>> = vec![vec![]; osid_to_graph_id.len()];
    let mut link_id = 0;
    for edge in edges {
        let angles = arrival_departure_angle_from_north(&edge.linestring);
        if edge.forward {
            add_forward_link(&mut driving_graph, &edge, &angles, time_group, &link_id, osid_to_graph_id);
            link_id += 1;
        }
        if edge.backward {
            add_backward_link(&mut driving_graph, &edge, &angles, time_group, &link_id, osid_to_graph_id);
            link_id += 1;
        }
    }
    driving_graph
}

pub fn add_forward_link(
    driving_graph: &mut Vec<Vec<(usize, usize, u16, u16, u32)>>,
    edge: &DrivingEdge,
    angles: &Angles,
    time_group: &str,
    link_id: &u32,
    osid_to_graph_id: &HashMap<OSNodeID, usize>,
) {
    // TODO: use precomputed length from OS source for unchanged edges
    let length = edge.linestring.haversine_length();
    let forward_traversal_time: usize = match time_group {
        "9to12" => calculate_traversal_time(edge.speeds.in9to12, length),
        "12to14" => calculate_traversal_time(edge.speeds.in12to14, length),
        "14to16" => calculate_traversal_time(edge.speeds.in14to16, length),
        "16to19" => calculate_traversal_time(edge.speeds.in16to19, length),
        _ => {
            panic!("Invalid time group");
        }
    };
    driving_graph[osid_to_graph_id[&edge.start_node]].push((
        forward_traversal_time,
        osid_to_graph_id[&edge.end_node],
        angles.forward_departure,
        angles.forward_arrival,
        *link_id,
    ));
}

pub fn add_backward_link(
    driving_graph: &mut Vec<Vec<(usize, usize, u16, u16, u32)>>,
    edge: &DrivingEdge,
    angles: &Angles,
    time_group: &str,
    link_id: &u32,
    osid_to_graph_id: &HashMap<OSNodeID, usize>,
) {
    let length = edge.linestring.haversine_length();
    let backward_traversal_time: usize = match time_group {
        "9to12" => calculate_traversal_time(edge.speeds.against9to12, length),
        "12to14" => calculate_traversal_time(edge.speeds.against12to14, length),
        "14to16" => calculate_traversal_time(edge.speeds.against14to16, length),
        "16to19" => calculate_traversal_time(edge.speeds.against16to19, length),
        _ => {
            panic!("Invalid time group");
        }
    };
    driving_graph[osid_to_graph_id[&edge.end_node]].push((
        backward_traversal_time,
        osid_to_graph_id[&edge.start_node],
        angles.backward_departure,
        angles.backward_arrival,
        *link_id,
    ));

}

fn calculate_traversal_time(
    speed: f64, //kph
    distance: f64, //meters
) -> usize {
    let time = distance / (speed * 1000.0 / 3600.0);
    time.round() as usize
}


#[derive(Debug)]
pub struct Angles {
    pub forward_arrival: u16,
    pub forward_departure: u16,
    pub backward_arrival: u16,
    pub backward_departure: u16,
}

pub fn arrival_departure_angle_from_north(linestring: &LineString) -> Angles {
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


