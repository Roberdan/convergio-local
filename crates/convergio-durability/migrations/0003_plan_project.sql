-- Add optional project/scope metadata to plans so dashboards can group
-- local work without overloading the title or description.
ALTER TABLE plans ADD COLUMN project TEXT;
