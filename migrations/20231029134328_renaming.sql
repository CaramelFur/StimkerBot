-- Rename sticker_tag to entity_tag
ALTER TABLE "sticker_tag" RENAME TO "entity_tag";
-- Rename sticker_stat to entity_stat
ALTER TABLE "sticker_stat" RENAME TO "entity_stat";

-- Rename entity_tag.sticker_id to entity_tag.entity_id
ALTER TABLE "entity_tag" RENAME COLUMN "sticker_id" TO "entity_id";
-- Rename entity_stat.sticker_id to entity_stat.entity_id
ALTER TABLE "entity_stat" RENAME COLUMN "sticker_id" TO "entity_id";
