-- Add nfc_present column to devices table
ALTER TABLE devices
ADD COLUMN nfc_present BOOLEAN NOT NULL DEFAULT FALSE;