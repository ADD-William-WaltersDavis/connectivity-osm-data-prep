#!/bin/bash

set -e

mkdir -p ../tmp
# Assume the root directory has the osm.pbf, used by many other scripts in this repo
time cargo run --release ../input/kent-240708.osm.pbf ../tmp/edges.json
