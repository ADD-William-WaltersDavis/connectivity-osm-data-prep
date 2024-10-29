#!/bin/bash

set -e

for year in 22 23 24; do 

    mkdir -p input
    wget https://download.geofabrik.de/europe/england-${year}0101.osm.pbf -O input/england-${year}0101.osm.pbf

    for mode in "walk" "cycling"; do
        mkdir -p tmp

        time cargo run --release input/england-${year}0101.osm.pbf tmp/england-${year}-${mode}.geojson ${mode}

        time tippecanoe tmp/england-${year}-${mode}.geojson \
            --force \
            --generate-ids \
            -l year \
            -zg \
            --drop-densest-as-needed \
            --extend-zooms-if-still-dropping \
            -o england-${year}-${mode}.mbtiles

        time ../../pmtiles convert england-${year}-${mode}.mbtiles england-20${year}-${mode}.pmtiles

        time gsutil -m cp england-20${year}-${mode}.pmtiles gs://very-nice-tiles-bucket/yearly-osm/england/20${year}-${mode}.pmtiles
    done
done