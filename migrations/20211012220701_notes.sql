CREATE TABLE IF NOT EXISTS crypton_notes (
	id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
	hash TEXT		NOT NULL,
	contents TEXT		NOT NULL,
	created_at DATETIME	NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX hash_index
ON crypton_notes(id, hash, contents);
