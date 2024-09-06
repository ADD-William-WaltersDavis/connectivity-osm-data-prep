#!/bin/bash

set -e

mkdir -p ../input

if [ ! -e "../input/kent-240708.osm.pbf" ]; then
	wget http://download.geofabrik.de/europe/great-britain/england/kent-240708.osm.pbf -P ../input
fi

mkdir -p ../data
# Assume the root directory has the osm.pbf, used by many other scripts in this repo
time cargo run --release ../input/kent-240708.osm.pbf ../data
