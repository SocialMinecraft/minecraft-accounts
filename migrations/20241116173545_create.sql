create table accounts (
    id bigserial primary key,
    user_id VARCHAR(100) NOT NULL,
    first_name VARCHAR(100) NOT NULl,
    minecraft_uuid VARCHAR(250) unique not null,
    minecraft_username VARCHAR(50) unique not null,
    is_main bool not null
);