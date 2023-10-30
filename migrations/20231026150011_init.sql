-- Add migration script here
CREATE TABLE IF NOT EXISTS "sticker_tag" (
  "sticker_id" text NOT NULL,
  "file_id" text NOT NULL,
  "user_id" text NOT NULL,
  "tag_name" text NOT NULL,
  PRIMARY KEY ("sticker_id", "user_id", "tag_name")
);

CREATE TABLE IF NOT EXISTS "sticker_stat" (
  "user_id" text NOT NULL,
  "sticker_id" text NOT NULL,
  "count" integer NOT NULL,
  PRIMARY KEY ("user_id", "sticker_id")
);
