CREATE TABLE user_entity_data (
  ue_id integer NOT NULL PRIMARY KEY AUTOINCREMENT,
  user_id text NOT NULL,
  entity_id text NOT NULL,
  created_at integer NOT NULL DEFAULT 0
);

ALTER TABLE entity_main RENAME TO entity_main_old;
CREATE TABLE IF NOT EXISTS entity_main (
  ue_id integer NOT NULL,
  tag_id text NOT NULL,
  PRIMARY KEY (ue_id, tag_id)
);

INSERT INTO user_entity_data (entity_id, user_id)
SELECT DISTINCT entity_id, user_id FROM entity_main_old;


INSERT INTO entity_main (ue_id, tag_id) SELECT ue_id, tag_id from (
  SELECT * FROM entity_main_old JOIN user_entity_data 
    ON entity_main_old.user_id = user_entity_data.user_id AND entity_main_old.entity_id = user_entity_data.entity_id
);

DROP TABLE entity_main_old;
