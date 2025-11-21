CREATE TABLE teams (
    "id" INTEGER PRIMARY KEY,
    "name" TEXT
);

CREATE TABLE employees (
    "id" INTEGER PRIMARY KEY,
    "name" TEXT,
    "team" INTEGER,
    FOREIGN KEY ("team") REFERENCES teams("id")
);