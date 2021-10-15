CREATE TABLE IF NOT EXISTS crypton_tags(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT UNIQUE NOT NULL
);
CREATE TABLE IF NOT EXISTS crypton_tag_mapping(
    tag_id INTEGER NOT NULL,
    note_id INTEGER NOT NULL,
    FOREIGN KEY (tag_id) REFERENCES crypton_tags(id)
	ON DELETE CASCADE,
    FOREIGN KEY (note_id) REFERENCES crypton_notes(id)
	ON DELETE CASCADE
);
INSERT INTO crypton_tags
VALUES (0, "footag");
INSERT INTO crypton_notes (id, hash, contents)
VALUES (0, "foohash", "foocontents");
INSERT INTO crypton_tag_mapping
VALUES (0, 0);