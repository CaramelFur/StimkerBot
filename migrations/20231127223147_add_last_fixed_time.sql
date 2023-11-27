-- Add user database with id, and last_fixed_time
-- Add index on id

CREATE TABLE user_data (
    user_id INTEGER PRIMARY KEY,
    last_fixed_time INTEGER DEFAULT 0
);
