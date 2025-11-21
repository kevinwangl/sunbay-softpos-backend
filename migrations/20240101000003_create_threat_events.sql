-- Create threat_events table
CREATE TABLE IF NOT EXISTS threat_events (
    id TEXT PRIMARY KEY NOT NULL,
    device_id TEXT NOT NULL,
    threat_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    status TEXT NOT NULL,
    description TEXT NOT NULL,
    detected_at TEXT NOT NULL,
    resolved_at TEXT,
    resolved_by TEXT,
    FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE CASCADE
);

-- Create indexes for threat_events table
CREATE INDEX IF NOT EXISTS idx_threat_events_device_id ON threat_events(device_id);
CREATE INDEX IF NOT EXISTS idx_threat_events_status ON threat_events(status);
CREATE INDEX IF NOT EXISTS idx_threat_events_severity ON threat_events(severity);
CREATE INDEX IF NOT EXISTS idx_threat_events_threat_type ON threat_events(threat_type);
CREATE INDEX IF NOT EXISTS idx_threat_events_detected_at ON threat_events(detected_at);
CREATE INDEX IF NOT EXISTS idx_threat_events_device_status ON threat_events(device_id, status);
