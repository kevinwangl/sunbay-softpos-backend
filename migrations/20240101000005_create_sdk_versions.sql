-- Create sdk_versions table
CREATE TABLE IF NOT EXISTS sdk_versions (
    id TEXT PRIMARY KEY NOT NULL,
    version TEXT NOT NULL UNIQUE,
    update_type TEXT NOT NULL,
    status TEXT NOT NULL,
    download_url TEXT NOT NULL,
    checksum TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    release_notes TEXT NOT NULL,
    min_os_version TEXT,
    target_devices TEXT,
    distribution_strategy TEXT,
    created_at TEXT NOT NULL,
    released_at TEXT
);

-- Create indexes for sdk_versions table
CREATE INDEX IF NOT EXISTS idx_sdk_versions_version ON sdk_versions(version);
CREATE INDEX IF NOT EXISTS idx_sdk_versions_status ON sdk_versions(status);
CREATE INDEX IF NOT EXISTS idx_sdk_versions_created_at ON sdk_versions(created_at);
