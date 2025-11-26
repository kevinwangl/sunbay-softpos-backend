#!/bin/bash

DB_PATH="./data/sunbay_softpos.db"

if [ ! -f "$DB_PATH" ]; then
    echo "Database file not found at $DB_PATH"
    exit 1
fi

echo "Clearing test data from $DB_PATH..."

sqlite3 "$DB_PATH" <<EOF
PRAGMA foreign_keys = OFF;
DELETE FROM transactions;
DELETE FROM health_checks;
DELETE FROM threat_events;
DELETE FROM audit_logs;
DELETE FROM devices;
DELETE FROM sdk_versions;
VACUUM;
EOF

echo "Test data cleared successfully."
