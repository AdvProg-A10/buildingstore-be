CREATE TABLE transaksi (
    id SERIAL PRIMARY KEY,
    id_pelanggan INTEGER NOT NULL,
    nama_pelanggan VARCHAR(255) NOT NULL,
    tanggal_transaksi VARCHAR(100) NOT NULL,
    total_harga DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    status VARCHAR(50) NOT NULL DEFAULT 'MASIH_DIPROSES',
    catatan TEXT,
    created_at VARCHAR(100) NOT NULL,
    updated_at VARCHAR(100) NOT NULL
)