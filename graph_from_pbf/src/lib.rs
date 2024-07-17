use geo::LineString;
use serde::Serialize;

#[derive(Serialize)]
pub struct Edge {
    pub id: usize,
    pub osm_id: i64,
    pub start_node: i64,
    pub end_node: i64,
    pub linestring: LineString,
}

#[derive(Serialize)]
pub struct Angles {
    pub forward_arrival: u16,
    pub forward_departure: u16,
    pub backward_arrival: u16,
    pub backward_departure: u16,
}
