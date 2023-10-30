-- Rename entity_tag to entity_tag_old
ALTER TABLE entity_tag RENAME TO entity_tag_old;

-- Create new entity_main table with only entity_id, user_id, tag_id
CREATE TABLE IF NOT EXISTS entity_main (
  entity_id text NOT NULL,
  user_id text NOT NULL,
  tag_id text NOT NULL,
  PRIMARY KEY (entity_id, user_id, tag_id)
);

-- Create new entity_file table with only entity_id, file_id, entity_type
CREATE TABLE IF NOT EXISTS entity_file (
  entity_id text NOT NULL PRIMARY KEY,
  file_id text NOT NULL,
  entity_type text NOT NULL
);

-- Create new entity_tag table with only tag_id, tag_name, autoincrement id
CREATE TABLE IF NOT EXISTS entity_tag (
  tag_id integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  tag_name text NOT NULL UNIQUE
);

-- Move data from entity_tag_old to entity_tag
INSERT INTO entity_tag (tag_name)
SELECT DISTINCT tag_name FROM entity_tag_old;

-- Move data from entity_tag_old to entity_file
INSERT INTO entity_file (entity_id, file_id, entity_type)
SELECT entity_id, file_id, entity_type FROM entity_tag_old GROUP BY entity_id;

-- Move data from entity_tag_old to entity_main
INSERT INTO entity_main (entity_id, user_id, tag_id)
SELECT entity_id, user_id, tag_id FROM (
  SELECT * from entity_tag_old
    JOIN entity_tag ON entity_tag_old.tag_name = entity_tag.tag_name
);

-- Drop entity_tag_old table
DROP TABLE IF EXISTS entity_tag_old;
