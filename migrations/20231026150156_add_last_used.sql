-- Add migration script here
-- Add last_used column to sticker_stat table
ALTER TABLE "sticker_stat"
ADD COLUMN IF NOT EXISTS "last_used" integer NOT NULL DEFAULT 0;
