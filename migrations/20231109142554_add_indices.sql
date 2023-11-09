-- Add migration script here
CREATE INDEX "entity_data_user_id" ON entity_data (
	"user_id"	ASC
);

CREATE INDEX "entity_data_entity_id" ON entity_data (
	"entity_id"	ASC
);

CREATE INDEX "entity_tag_tag_name" ON entity_tag (
  "tag_name"	ASC
);