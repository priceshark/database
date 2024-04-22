stores:
    #!/bin/sh
    cd stores/raw
    [ -e coles-osm.json ] || curl https://overpass-api.de/api/interpreter --data @coles-osm.overpassql > coles-osm.json
    [ -e woolworths-osm.json ] || curl https://overpass-api.de/api/interpreter --data @woolworths-osm.overpassql > woolworths-osm.json
