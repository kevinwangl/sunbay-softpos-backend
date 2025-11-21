-- Create devices table
CREATE TABLE IF NOT EXISTS devices (
    id TEXT PRIMARY KEY NOT NULL,
    imei TEXT NOT NULL UNIQUE,
    model TEXT NOT NULL,
    os_version TEXT NOT NULL,
    tee_type TEXT NOT NULL CHECK(tee_type IN ('QTEE', 'TRUSTZONE')),
    device_mode TEXT NOT NULL DEFAULT 'FULL_POS' CHECK(device_mode IN ('FULL_POS', 'PINPAD')),
    public_key BLOB NOT NULL,
    status TEXT NOT NULL DEFAULT 'PENDING' CHECK(status IN ('PENDING', 'ACTIVE', 'SUSPENDED', 'REVOKED', 'REJECTED')),
    merchant_id TEXT,
    merchant_name TEXT,
    security_score INTEGER NOT NULL DEFAULT 0 CHECK(security_score >= 0 AND security_score <= 100),
    current_ksn TEXT NOT NULL,
    ipek_injected_at TEXT,
    key_remaining_count INTEGER NOT NULL DEFAULT 1000000,
    key_total_count INTEGER NOT NULL DEFAULT 1000000,
    registered_at TEXT NOT NULL,
    approved_at TEXT,
    approved_by TEXT,
    last_active_at TEXT,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for devices table
CREATE INDEX IF NOT EXISTS idx_devices_status ON devices(status);
CREATE INDEX IF NOT EXISTS idx_devices_merchant_id ON devices(merchant_id);
CREATE INDEX IF NOT EXISTS idx_devices_security_score ON devices(security_score);
CREATE INDEX IF NOT EXISTS idx_devices_imei ON devices(imei);
CREATE INDEX IF NOT EXISTS idx_devices_registered_at ON devices(registered_at);
CREATE INDEX IF NOT EXISTS idx_devices_last_active_at ON devices(last_active_at);
