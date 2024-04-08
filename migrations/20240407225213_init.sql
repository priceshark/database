create table raw_price (
    retailer integer not null,
    store integer not null,
    product integer not null,
    price float,
    discount_price float,
    discount_member_only boolean not null,
    discount_online_only boolean not null,
    discount_quantity integer not null,
    promotion integer not null,
    primary key (retailer, store, product)
);

create table raw_price_history (
    timestamp integer not null,
    retailer integer not null,
    store integer not null,
    product integer not null,
    price float,
    discount_price float,
    discount_member_only boolean not null,
    discount_online_only boolean not null,
    discount_quantity integer not null,
    promotion integer not null
    -- no primary key due to index size
);
