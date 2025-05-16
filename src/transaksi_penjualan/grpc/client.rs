use tonic::transport::{Channel, Endpoint};
use log::{info, error};

use crate::grpc::proto::{
    TransaksiServiceClient,
    TransaksiRequest,
    TransaksiId,
    TransaksiFilter,
    UpdateStatusRequest,
};

pub struct TransaksiClient {
    client: TransaksiServiceClient<Channel>,
}

impl TransaksiClient {
    pub async fn connect(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let endpoint = Endpoint::from_shared(format!("http://{}", addr))?;
        let channel = endpoint.connect().await?;
        let client = TransaksiServiceClient::new(channel);

        Ok(Self { client })
    }

    pub async fn create_transaksi(
        &mut self,
        request: TransaksiRequest,
    ) -> Result<tonic::Response<crate::grpc::proto::TransaksiResponse>, tonic::Status> {
        self.client.create_transaksi(request).await
    }

    pub async fn get_transaksi(
        &mut self,
        id: String,
    ) -> Result<tonic::Response<crate::grpc::proto::TransaksiResponse>, tonic::Status> {
        let request = TransaksiId { id };
        self.client.get_transaksi(request).await
    }

    pub async fn list_transaksi(
        &mut self,
        filter: TransaksiFilter,
    ) -> Result<tonic::Response<crate::grpc::proto::TransaksiList>, tonic::Status> {
        self.client.list_transaksi(filter).await
    }

    pub async fn update_transaksi_status(
        &mut self,
        id: String,
        status: String,
    ) -> Result<tonic::Response<crate::grpc::proto::TransaksiResponse>, tonic::Status> {
        let request = UpdateStatusRequest { id, status };
        self.client.update_transaksi_status(request).await
    }

    pub async fn cancel_transaksi(
        &mut self,
        id: String,
    ) -> Result<tonic::Response<crate::grpc::proto::TransaksiResponse>, tonic::Status> {
        let request = TransaksiId { id };
        self.client.cancel_transaksi(request).await
    }
}