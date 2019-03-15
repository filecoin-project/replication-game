-- Your SQL goes here

ALTER TABLE params
ADD COLUMN is_tapered BOOLEAN,
ADD COLUMN taper_layers INT,
ADD COLUMN taper DOUBLE PRECISION;
