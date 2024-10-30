#!/bin/bash

set -e

for year in 14 15 16 17 18 19 20 21 22 23 24; do 

    mkdir -p input
    wget https://download.geofabrik.de/europe/united-kingdom/england-${year}0101.osm.pbf -O input/england-${year}0101.osm.pbf

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
            -o tmp/england-${year}-${mode}.mbtiles

        time ../../pmtiles convert tmp/england-${year}-${mode}.mbtiles tmp/england-20${year}-${mode}.pmtiles

        time gsutil -m cp tmp/england-20${year}-${mode}.pmtiles gs://very-nice-tiles-bucket/yearly-osm/england/20${year}-${mode}.pmtiles
        
        rm tmp/england-${year}-${mode}.geojson
        rm tmp/england-${year}-${mode}.mbtiles
        rm tmp/england-20${year}-${mode}.pmtiles
    done
done