CREATE TABLE detail_transaksi (
    id SERIAL PRIMARY KEY,
    id_transaksi INTEGER NOT NULL,
    id_produk INTEGER NOT NULL,
    harga_satuan DECIMAL(15,2) NOT NULL,
    jumlah INTEGER NOT NULL,
    subtotal DECIMAL(15,2) NOT NULL,
    created_at VARCHAR(100),
    updated_at VARCHAR(100),
    FOREIGN KEY (id_transaksi) REFERENCES transaksi(id) ON DELETE CASCADE
)