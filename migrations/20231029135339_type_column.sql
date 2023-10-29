-- Add text column called type to entity_tag table
ALTER TABLE "entity_tag"
ADD COLUMN "entity_type" text NOT NULL DEFAULT 'sticker';

