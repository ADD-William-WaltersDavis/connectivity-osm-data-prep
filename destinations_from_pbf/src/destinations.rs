use crate::*;
use destinations_from_pbf::Destination;
use std::collections::HashMap;

use geo::{Centroid, Coord, Geometry, LineString, Point, Polygon};
use indicatif::{ProgressBar, ProgressStyle};
use osm_reader::{Element, NodeID};

pub fn process(osm_path: &str) -> Result<Vec<Destination>> {
    let destinations = scrape_osm(osm_path)?;
    Ok(destinations)
}

fn scrape_osm(osm_path: &str) -> Result<Vec<Destination>> {
    let mut node_mapping: HashMap<NodeID, Coord> = HashMap::new();
    let mut destinations: Vec<Destination> = Vec::new();
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
        Element::Node {
            id, lon, lat, tags, ..
        } => {
            nodes_progress.inc(1);
            let coord = Coord { x: lon, y: lat };
            node_mapping.insert(id, coord);
            if tags.get("amenity") == Some(&"hospital".to_string())
                || tags.get("amenity") == Some(&"doctors".to_string())
                || tags.get("amenity") == Some(&"clinic".to_string())
                || tags.get("amenity") == Some(&"pharmacy".to_string())
            {
                // let name be the tags name unless not present then use the id
                let name = tags.get("name").unwrap_or(&id.0.to_string()).to_string();
                let purpose = "Health".to_string();
                let subpurpose = tags.get("amenity").unwrap();
                destinations.push(Destination {
                    id: id.0 as usize,
                    name: name.clone(),
                    purpose: purpose.clone(),
                    subpurpose: subpurpose.clone(),
                    geometry: Geometry::Point(Point(coord)),
                    centroid: Point(coord),
                });
            }
        }
        Element::Way {
            id, node_ids, tags, ..
        } => {
            if tags.get("amenity") == Some(&"hospital".to_string())
                || tags.get("amenity") == Some(&"doctors".to_string())
                || tags.get("amenity") == Some(&"clinic".to_string())
                || tags.get("amenity") == Some(&"pharmacy".to_string())
            {
                // TODO: add oneway tag filtering here
                if first_way {
                    nodes_progress.finish();
                    first_way = false;
                }
                ways_progress.inc(1);
                let name = tags.get("name").unwrap_or(&id.0.to_string()).to_string();
                let purpose = "Health".to_string();
                let subpurpose = tags.get("amenity").unwrap();
                let mut coords = Vec::new();
                for node_id in node_ids {
                    if let Some(coord) = node_mapping.get(&node_id) {
                        coords.push(*coord);
                    }
                }
                let polygon = Polygon::new(LineString(coords), vec![]);
                let centroid = polygon.centroid().unwrap();
                destinations.push(Destination {
                    id: id.0 as usize,
                    name: name.clone(),
                    purpose: purpose.clone(),
                    subpurpose: subpurpose.clone(),
                    geometry: Geometry::Polygon(polygon),
                    centroid: centroid,
                });
            }
        }
        Element::Relation { .. } | Element::Bounds { .. } => {}
    })?;
    ways_progress.finish();

    Ok(destinations)
}
