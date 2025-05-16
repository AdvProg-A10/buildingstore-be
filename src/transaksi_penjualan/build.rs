fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=transaksi_penjualan/proto/transaksi.proto");
    tonic_build::compile_protos("transaksi_penjualan/proto/transaksi.proto")?;
    Ok(())
}