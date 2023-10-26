-- Add migration script here
CREATE TABLE "sticker_tag" IF NOT EXISTS (
  "sticker_id" text NOT NULL,
  "file_id" text NOT NULL,
  "user_id" text NOT NULL,
  "tag_name" text NOT NULL,
  PRIMARY KEY ("sticker_id", "user_id", "tag_name")
);

CREATE TABLE "sticker_stat" IF NOT EXISTS (
  "user_id" text NOT NULL,
  "sticker_id" text NOT NULL,
  "count" integer NOT NULL,
  PRIMARY KEY ("user_id", "sticker_id")
);
