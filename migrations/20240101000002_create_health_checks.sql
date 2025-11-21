-- Create health_checks table
CREATE TABLE IF NOT EXISTS health_checks (
    id TEXT PRIMARY KEY NOT NULL,
    device_id TEXT NOT NULL,
    security_score INTEGER NOT NULL CHECK(security_score >= 0 AND security_score <= 100),
    root_status INTEGER NOT NULL CHECK(root_status IN (0, 1)),
    bootloader_status INTEGER NOT NULL CHECK(bootloader_status IN (0, 1)),
    system_integrity INTEGER NOT NULL CHECK(system_integrity IN (0, 1)),
    app_integrity INTEGER NOT NULL CHECK(app_integrity IN (0, 1)),
    tee_status INTEGER NOT NULL CHECK(tee_status IN (0, 1)),
    recommended_action TEXT NOT NULL CHECK(recommended_action IN ('None', 'Monitor', 'Suspend', 'Revoke')),
    details TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE CASCADE
);

-- Create indexes for health_checks table
CREATE INDEX IF NOT EXISTS idx_health_checks_device_id ON health_checks(device_id);
CREATE INDEX IF NOT EXISTS idx_health_checks_created_at ON health_checks(created_at);
CREATE INDEX IF NOT EXISTS idx_health_checks_security_score ON health_checks(security_score);
CREATE INDEX IF NOT EXISTS idx_health_checks_device_created_at ON health_checks(device_id, created_at DESC);
