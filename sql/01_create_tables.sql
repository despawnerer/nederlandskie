CREATE TABLE IF NOT EXISTS Post (
    id INT GENERATED ALWAYS AS IDENTITY,
    indexed_at TIMESTAMP WITH TIME ZONE,
    cid TEXT UNIQUE,
    uri TEXT UNIQUE,
    author_did TEXT
);
