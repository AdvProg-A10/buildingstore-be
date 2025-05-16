pub mod server;
pub mod client;
pub mod transaksi_service_impl;

pub use server::TransaksiServer;
pub use client::TransaksiClient;

pub mod proto {
    tonic::include_proto!("transaksi_penjualan");
    
    pub use self::transaksi_service_server::{TransaksiService, TransaksiServiceServer};
    pub use self::transaksi_service_client::TransaksiServiceClient;
}