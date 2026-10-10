-- Chasing is offline: there is no email, so the app's job is to make the
-- reminder easy to send, and to remember that it went.
ALTER TABLE invoice ADD COLUMN last_chased_at TEXT;
ALTER TABLE invoice ADD COLUMN chase_count INTEGER NOT NULL DEFAULT 0;
