-- Ledgerly is free and open source, with no Pro tier, so the licensing tables
-- are dropped.
--
-- They are dropped by a new migration rather than by removing 0006 from the
-- list: migrations are append-only. Deleting an entry would shift every later
-- version number and strand databases that are already past it.
DROP TABLE IF EXISTS trial;
DROP TABLE IF EXISTS license;
DROP TABLE IF EXISTS installation;
