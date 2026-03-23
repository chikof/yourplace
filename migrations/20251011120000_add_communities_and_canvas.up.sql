CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS communities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    width INT NOT NULL DEFAULT 512,
    height INT NOT NULL DEFAULT 512,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    billing_plan TEXT NOT NULL DEFAULT 'free',
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS community_members (
    community_id UUID NOT NULL REFERENCES communities(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'member',
    joined_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (community_id, user_id)
);

ALTER TABLE users ALTER COLUMN id SET DEFAULT gen_random_uuid();

ALTER TABLE pixels ADD COLUMN IF NOT EXISTS community_id UUID REFERENCES communities(id) ON DELETE CASCADE;
ALTER TABLE pixels DROP CONSTRAINT IF EXISTS pixel_pkey;
ALTER TABLE pixels ADD CONSTRAINT pixel_pkey PRIMARY KEY (community_id, x, y);
CREATE INDEX IF NOT EXISTS idx_pixels_community ON pixels (community_id);
