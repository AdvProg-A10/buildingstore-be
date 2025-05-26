CREATE TABLE IF NOT EXISTS payments(
    id TEXT PRIMARY KEY,
    transaction_id TEXT NOT NULL,
    amount REAL NOT NULL,
    method TEXT NOT NULL,
    status TEXT NOT NULL,
    payment_date TEXT NOT NULL,
    due_date TEXT
);

CREATE INDEX IF NOT EXISTS idx_payments_transaction_id ON payments(transaction_id);
CREATE INDEX IF NOT EXISTS idx_payments_status ON payments(status);
CREATE INDEX IF NOT EXISTS idx_payments_method ON payments(method);