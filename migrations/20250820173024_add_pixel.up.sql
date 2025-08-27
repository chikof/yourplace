-- Add up migration script here
CREATE TABLE IF NOT EXISTS pixels (
    x INT NOT NULL,
    y INT NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    placed_at TIMESTAMP NOT NULL DEFAULT NOW(),
    color INT NOT NULL,

    CONSTRAINT pixel_pkey PRIMARY KEY (x, y)
);
