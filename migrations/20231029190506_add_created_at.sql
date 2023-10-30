CREATE TABLE entity_data (
  combo_id integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  user_id text NOT NULL,
  entity_id text NOT NULL,

  count integer NOT NULL DEFAULT 0,
  created_at integer NOT NULL DEFAULT 0,
  last_used integer NOT NULL DEFAULT 0,
  CONSTRAINT entity_data_unique UNIQUE (user_id, entity_id)
);

ALTER TABLE entity_main RENAME TO entity_main_old;
CREATE TABLE IF NOT EXISTS entity_main (
  combo_id integer NOT NULL,
  tag_id text NOT NULL,
  PRIMARY KEY (combo_id, tag_id)
);

INSERT INTO entity_data (entity_id, user_id, count, last_used)
SELECT entity_id, user_id, count, last_used FROM entity_stat;

DROP TABLE entity_stat;

INSERT OR IGNORE INTO entity_data (entity_id, user_id)
SELECT DISTINCT entity_id, user_id FROM entity_main_old;

INSERT INTO entity_main (combo_id, tag_id) SELECT combo_id, tag_id from (
  SELECT * FROM entity_main_old JOIN entity_data 
    ON entity_main_old.user_id = entity_data.user_id AND entity_main_old.entity_id = entity_data.entity_id
);

DROP TABLE entity_main_old;