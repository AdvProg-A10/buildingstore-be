use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;
use log::{info, error};

use crate::main::service::transaksi_service::TransaksiService;
use crate::grpc::transaksi_service_impl::TransaksiServiceImpl;
use crate::grpc::proto::TransaksiServiceServer;

pub struct TransaksiServer {
    service: Arc<TransaksiService>,
    addr: SocketAddr,
}

impl TransaksiServer {
    pub fn new(service: Arc<TransaksiService>, addr: SocketAddr) -> Self {
        Self { service, addr }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let service = TransaksiServiceImpl::new(Arc::clone(&self.service));
        let server = TransaksiServiceServer::new(service);

        info!("TransaksiServer listening on {}", self.addr);

        Server::builder()
            .add_service(server)
            .serve(self.addr)
            .await?;

        Ok(())
    }

    pub async fn start_with_shutdown_signal(
        &self, 
        shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static
    ) -> Result<(), Box<dyn std::error::Error>> {
        let service = TransaksiServiceImpl::new(Arc::clone(&self.service));
        let server = TransaksiServiceServer::new(service);
    
        info!("TransaksiServer listening on {}", self.addr);
    
        Server::builder()
            .add_service(server)
            .serve_with_shutdown(self.addr, shutdown_signal)
            .await?;
    
        Ok(())
    }
}