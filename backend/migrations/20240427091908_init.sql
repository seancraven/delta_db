-- Add migration script here
CREATE TABLE BaseCfgs(
    cfg_hash TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    cfg JSON NOT NULL,
    version INTEGER NOT NULL,
    UNIQUE(name, version)
);

CREATE TABLE Deltas (
    id INTEGER NOT NULL PRIMARY KEY,
    cfg_hash TEXT NOT NULL,
    delta JSON NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY(cfg_hash) REFERENCES BaseCfgs(cfg_hash)
);
