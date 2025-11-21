-- Create audit_logs table
CREATE TABLE IF NOT EXISTS audit_logs (
    id TEXT PRIMARY KEY NOT NULL,
    operation TEXT NOT NULL,
    operator TEXT NOT NULL,
    device_id TEXT,
    result TEXT NOT NULL CHECK(result IN ('Success', 'Failure', 'Partial')),
    details TEXT,
    ip_address TEXT,
    user_agent TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE SET NULL
);

-- Create indexes for audit_logs table
CREATE INDEX IF NOT EXISTS idx_audit_logs_operator ON audit_logs(operator);
CREATE INDEX IF NOT EXISTS idx_audit_logs_device_id ON audit_logs(device_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_audit_logs_operation ON audit_logs(operation);
CREATE INDEX IF NOT EXISTS idx_audit_logs_result ON audit_logs(result);
CREATE INDEX IF NOT EXISTS idx_audit_logs_operator_created_at ON audit_logs(operator, created_at DESC);
