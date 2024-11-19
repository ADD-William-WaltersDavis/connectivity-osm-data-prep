#!/bin/bash

set -e

for year in 14 15 16 17 18 19 20 21 22 23 24; do 

    mkdir -p input
    for country in "england" "wales" "scotland"; do
        wget https://download.geofabrik.de/europe/united-kingdom/${country}-${year}0101.osm.pbf -O input/${country}-${year}0101.osm.pbf
    done

    for mode in "walk" "cycling"; do
        mkdir -p tmp
        time cargo run --release input/england-${year}0101.osm.pbf input/wales-${year}0101.osm.pbf input/scotland-${year}0101.osm.pbf tmp/gb-${year}-${mode}.geojson ${mode}

        time tippecanoe tmp/gb-${year}-${mode}.geojson \
            --force \
            --generate-ids \
            -l year \
            -zg \
            --drop-densest-as-needed \
            --extend-zooms-if-still-dropping \
            -o tmp/gb-${year}-${mode}.mbtiles

        time ../../pmtiles convert tmp/gb-${year}-${mode}.mbtiles tmp/gb-20${year}-${mode}.pmtiles

        time gsutil -m cp tmp/gb-20${year}-${mode}.pmtiles gs://very-nice-tiles-bucket/yearly-osm/gb/20${year}-${mode}.pmtiles
        
        rm tmp/gb-${year}-${mode}.geojson
        rm tmp/gb-${year}-${mode}.mbtiles
        rm tmp/gb-20${year}-${mode}.pmtiles
    done
    # Remove the osm.pbf files to save space
	for country in "england" "wales" "scotland"; do
		if [ ! -e "input/${country}-${year}0101.osm.pbf" ]; then
			rm input/${country}-${year}0101.osm.pbf
		fi
	done
done