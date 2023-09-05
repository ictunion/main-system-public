ALTER TABLE registration_requests
ALTER COLUMN registration_local SET NOT NULL;

-- Fix typo
ALTER VIEW registration_requests_procession RENAME TO registration_requests_processing;
