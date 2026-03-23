DROP INDEX IF EXISTS idx_pixels_community;
ALTER TABLE pixels DROP CONSTRAINT IF EXISTS pixel_pkey;
ALTER TABLE pixels DROP COLUMN IF EXISTS community_id;
ALTER TABLE pixels ADD CONSTRAINT pixel_pkey PRIMARY KEY (x, y);

DROP TABLE IF EXISTS community_members;
DROP TABLE IF EXISTS communities;
