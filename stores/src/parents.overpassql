[out:json][timeout:25];

is_in(LOCATION) ->.b;
(
	area(pivot.b)["building"];
	relation(pivot.b)["building"];
	area(pivot.b)["landuse"="retail"];
	relation(pivot.b)["landuse"="retail"];
);

out tags center;
