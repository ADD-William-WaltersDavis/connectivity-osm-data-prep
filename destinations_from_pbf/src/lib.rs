use geo::Geometry;
use serde::Serialize;

#[derive(Serialize)]
pub struct Destination {
    pub id: usize,
    pub name: String,
    pub purpose: String,
    pub subpurpose: String,
    pub geometry: Geometry,
}
