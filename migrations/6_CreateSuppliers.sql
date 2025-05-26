CREATE TABLE if not EXISTS suppliers (
    id VARCHAR(255) PRIMARY KEY NOT NULL,
    name VARCHAR(255) NOT NULL,
    jenis_barang VARCHAR(255),
    jumlah_barang INTEGER NOT NULL DEFAULT 0,
    resi VARCHAR(255),
    updated_at TEXT NOT NULL
);