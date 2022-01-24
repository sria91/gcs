-- Your SQL goes here
CREATE TABLE IF NOT EXISTS plateaus (
    id VARCHAR NOT NULL PRIMARY KEY NOT NULL,
    created_at TIMESTAMP NOT NULL,
    x_max INTEGER NOT NULL,
    y_max INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS rovers (
    id VARCHAR NOT NULL PRIMARY KEY NOT NULL,
    created_at TIMESTAMP NOT NULL,
    x INTEGER NOT NULL,
    y INTEGER NOT NULL,
    facing TEXT CHECK(facing IN ('north', 'east', 'south', 'west')) NOT NULL,
    plateau_id VARCHAR NOT NULL,
    FOREIGN KEY(plateau_id) REFERENCES plateaus(id)
);
