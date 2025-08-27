-- Add up migration script here
CREATE TABLE IF NOT EXISTS user_avatars (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    pixels JSONB NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT user_avatar_pkey PRIMARY KEY (user_id)
);
