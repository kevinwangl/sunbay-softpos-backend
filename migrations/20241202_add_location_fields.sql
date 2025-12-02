-- 添加交易位置信息字段
-- 2024-12-02
ALTER TABLE transactions
ADD COLUMN client_ip VARCHAR(45);
ALTER TABLE transactions
ADD COLUMN latitude DECIMAL(10, 8);
ALTER TABLE transactions
ADD COLUMN longitude DECIMAL(11, 8);
ALTER TABLE transactions
ADD COLUMN location_accuracy REAL;
ALTER TABLE transactions
ADD COLUMN location_timestamp TIMESTAMP;
-- 添加索引以提高查询性能
CREATE INDEX idx_transactions_client_ip ON transactions(client_ip);
CREATE INDEX idx_transactions_location ON transactions(latitude, longitude);