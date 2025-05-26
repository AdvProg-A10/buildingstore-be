CREATE TABLE detail_transaksi (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    id_transaksi INTEGER NOT NULL,
    id_produk INTEGER NOT NULL,
    harga_satuan REAL NOT NULL,
    jumlah INTEGER NOT NULL,
    subtotal REAL NOT NULL,
    created_at VARCHAR(100) NOT NULL,
    updated_at VARCHAR(100) NOT NULL,
    FOREIGN KEY (id_transaksi) REFERENCES transaksi(id) ON DELETE CASCADE
)