CREATE TABLE IF NOT EXISTS installments (
    id TEXT PRIMARY KEY,
    payment_id TEXT NOT NULL,
    amount REAL NOT NULL,
    payment_date TEXT NOT NULL,
    FOREIGN KEY (payment_id) REFERENCES payments(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_installments_payment_id ON installments(payment_id);