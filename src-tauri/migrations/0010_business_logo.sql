-- The logo lives in the database rather than on disk: the app promises one local
-- file, and a backup is a copy of this database, so a logo sitting beside it
-- would be the one thing a backup silently left behind. A separate table keeps
-- the bytes out of every business::get call.
CREATE TABLE business_logo (
    id    TEXT PRIMARY KEY CHECK (id = 'default'),
    bytes BLOB NOT NULL,
    mime  TEXT NOT NULL
);
