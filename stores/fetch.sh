#!/bin/sh

curl https://overpass-api.de/api/interpreter --data @coles-osm.overpassql > coles-osm-raw.json
curl https://overpass-api.de/api/interpreter --data @woolworths-osm.overpassql > woolworths-osm-raw.json
