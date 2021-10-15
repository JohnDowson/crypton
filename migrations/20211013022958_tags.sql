CREATE TABLE IF NOT EXISTS crypton_tags(
id INTEGER UNIQUE NOT NULL PRIMARY KEY,
name TEXT
);
CREATE TABLE IF NOT EXISTS crypton_tag_mapping(
tag_id INTEGER,
note_id INTEGER,
FOREIGN KEY (tag_id) REFERENCES crypton_tags(id),
FOREIGN KEY (note_id) REFERENCES crypton_notes(id)
)
