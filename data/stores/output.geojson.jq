{
  "type": "FeatureCollection",
  "features": map(
    {
      "type": "Feature",
      "properties": .,
      "geometry": {
        "type": "Point",
        "coordinates": [ .y, .x ]
      }
    })
}

