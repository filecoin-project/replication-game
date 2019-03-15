-- This file should undo anything in `up.sql`

ALTER TABLE params
DROP COLUMN is_tapered,
DROP COLUMN taper_layers,
DROP COLUMN taper;
