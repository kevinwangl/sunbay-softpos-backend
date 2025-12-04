-- Create kernels table
CREATE TABLE IF NOT EXISTS kernels (
    id TEXT PRIMARY KEY,
    version TEXT NOT NULL UNIQUE,
    file_path TEXT NOT NULL,
    file_hash TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
-- Create index on version for faster lookups
CREATE INDEX IF NOT EXISTS idx_kernels_version ON kernels(version);
-- Create index on status for filtering
CREATE INDEX IF NOT EXISTS idx_kernels_status ON kernels(status);