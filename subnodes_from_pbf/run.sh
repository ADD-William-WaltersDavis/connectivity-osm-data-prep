#!/bin/bash

set -e

mkdir -p ../input
if [ ! -e "../input/UK-dem-50m-4326.tif" ]; then
	wget https://play.abstreet.org/dev/data/input/shared/elevation/UK-dem-50m-4326.tif.gz -P ../input
	gunzip ../input/UK-dem-50m-4326.tif.gz
fi

for country in "england" "wales" "scotland"; do
	if [ ! -e "../input/${country}-240901.osm.pbf" ]; then
		wget https://download.geofabrik.de/europe/united-kingdom/${country}-240901.osm.pbf -O ../input/${country}-240901.osm.pbf
	fi
done

mkdir -p ../data

time cargo run --release \
	../input/england-240901.osm.pbf \
	../input/wales-240901.osm.pbf \
	../input/scotland-240901.osm.pbf \
	../input/UK-dem-50m-4326.tif \
	../data \
