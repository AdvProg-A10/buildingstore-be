use std::sync::Arc;
use std::pin::Pin;
use tonic::{Request, Response, Status};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use log::{info, error, debug};
use tokio::sync::mpsc;
use tokio_stream::Stream;
use futures::StreamExt;

use crate::main::service::transaksi_service::TransaksiService;
use crate::main::model::transaksi::{Transaksi, TransaksiItem};
use crate::main::enums::status_transaksi::StatusTransaksi;
use crate::main::observer::Observer;
use crate::grpc::proto::{
    self,
    TransaksiRequest,
    TransaksiResponse,
    TransaksiId,
    TransaksiFilter,
    TransaksiList,
    UpdateStatusRequest,
    ItemTransaksi,
    TransaksiEvent,
    transaksi_event,
};

pub struct TransaksiServiceImpl {
    service: Arc<TransaksiService>,
}

impl TransaksiServiceImpl {
    pub fn new(service: Arc<TransaksiService>) -> Self {
        Self { service }
    }

    fn transaksi_to_response(&self, transaksi: &Transaksi) -> TransaksiResponse {
        let mut items = Vec::new();
        
        for item in &transaksi.items {
            items.push(ItemTransaksi {
                product_id: item.product_id.clone(),
                quantity: item.quantity as i32,
                harga_satuan: item.harga_satuan,
                diskon: item.diskon,
            });
        }

        TransaksiResponse {
            id: transaksi.id.to_string(),
            customer_id: transaksi.customer_id.clone(),
            customer_name: transaksi.customer_name.clone().unwrap_or_default(),
            items,
            total_harga: transaksi.total_harga,
            total_diskon: transaksi.total_diskon,
            total_bayar: transaksi.total_bayar,
            metode_pembayaran: transaksi.metode_pembayaran.clone(),
            status: transaksi.status.to_string(),
            catatan: transaksi.catatan.clone().unwrap_or_default(),
            created_at: Some(prost_types::Timestamp::from(transaksi.created_at)),
            updated_at: Some(prost_types::Timestamp::from(transaksi.updated_at)),
        }
    }

    fn request_to_transaksi(&self, request: &TransaksiRequest) -> Transaksi {
        let mut items = Vec::new();
        
        for item in &request.items {
            items.push(TransaksiItem {
                product_id: item.product_id.clone(),
                quantity: item.quantity as u32,
                harga_satuan: item.harga_satuan,
                diskon: item.diskon,
            });
        }

        Transaksi {
            id: Uuid::new_v4(),
            customer_id: request.customer_id.clone(),
            customer_name: if request.customer_id.is_empty() { None } else { Some(String::new()) },
            items,
            total_harga: 0.0,
            total_diskon: 0.0,
            total_bayar: 0.0,
            metode_pembayaran: request.metode_pembayaran.clone(),
            status: StatusTransaksi::Pending,
            catatan: if request.catatan.is_empty() { None } else { Some(request.catatan.clone()) },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[tonic::async_trait]
impl proto::TransaksiService for TransaksiServiceImpl {
    async fn create_transaksi(
        &self,
        request: Request<TransaksiRequest>,
    ) -> Result<Response<TransaksiResponse>, Status> {
        let req = request.into_inner();
        debug!("Received CreateTransaksi request: {:?}", req);
        
        let transaksi_req = self.request_to_transaksi(&req);
        
        match self.service.create_transaksi(transaksi_req).await {
            Ok(transaksi) => {
                let response = self.transaksi_to_response(&transaksi);
                Ok(Response::new(response))
            },
            Err(e) => {
                error!("Failed to create transaksi: {}", e);
                Err(Status::internal(format!("Failed to create transaksi: {}", e)))
            }
        }
    }

    async fn get_transaksi(
        &self,
        request: Request<TransaksiId>,
    ) -> Result<Response<TransaksiResponse>, Status> {
        let id = request.into_inner().id;
        debug!("Received GetTransaksi request for id: {}", id);
        
        let uuid = match Uuid::parse_str(&id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(Status::invalid_argument("Invalid UUID format")),
        };
        
        match self.service.get_transaksi(uuid).await {
            Ok(Some(transaksi)) => {
                let response = self.transaksi_to_response(&transaksi);
                Ok(Response::new(response))
            },
            Ok(None) => {
                Err(Status::not_found(format!("Transaksi with id {} not found", id)))
            },
            Err(e) => {
                error!("Failed to get transaksi: {}", e);
                Err(Status::internal(format!("Failed to get transaksi: {}", e)))
            }
        }
    }

    async fn list_transaksi(
        &self,
        request: Request<TransaksiFilter>,
    ) -> Result<Response<TransaksiList>, Status> {
        let filter = request.into_inner();
        debug!("Received ListTransaksi request with filter: {:?}", filter);
        
        let page = filter.page.max(1) as u32;
        let limit = filter.limit.max(1) as u32;
        
        let tanggal_mulai = filter.tanggal_mulai.map(|ts| {
            DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32).unwrap_or_default()
        });
        
        let tanggal_akhir = filter.tanggal_akhir.map(|ts| {
            DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32).unwrap_or_default()
        });
        
        let status = if filter.status.is_empty() {
            None
        } else {
            match StatusTransaksi::from_string(&filter.status) {
                Ok(status) => Some(status),
                Err(_) => return Err(Status::invalid_argument("Invalid status value")),
            }
        };

        match self.service.list_transaksi(
            if filter.customer_id.is_empty() { None } else { Some(filter.customer_id) },
            tanggal_mulai,
            tanggal_akhir,
            status,
            page,
            limit
        ).await {
            Ok((transaksis, total_count)) => {
                let responses: Vec<TransaksiResponse> = transaksis
                    .iter()
                    .map(|t| self.transaksi_to_response(t))
                    .collect();
                
                let response = TransaksiList {
                    transaksi: responses,
                    total_count: total_count as i32,
                    page: page as i32,
                    limit: limit as i32,
                };
                
                Ok(Response::new(response))
            },
            Err(e) => {
                error!("Failed to list transaksi: {}", e);
                Err(Status::internal(format!("Failed to list transaksi: {}", e)))
            }
        }
    }

    async fn update_transaksi_status(
        &self,
        request: Request<UpdateStatusRequest>,
    ) -> Result<Response<TransaksiResponse>, Status> {
        let req = request.into_inner();
        debug!("Received UpdateTransaksiStatus request: {:?}", req);
        
        let uuid = match Uuid::parse_str(&req.id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(Status::invalid_argument("Invalid UUID format")),
        };
        
        let status = match StatusTransaksi::from_string(&req.status) {
            Ok(status) => status,
            Err(_) => return Err(Status::invalid_argument("Invalid status value")),
        };
        
        match self.service.update_status(uuid, status).await {
            Ok(transaksi) => {
                let response = self.transaksi_to_response(&transaksi);
                Ok(Response::new(response))
            },
            Err(e) => {
                error!("Failed to update transaksi status: {}", e);
                Err(Status::internal(format!("Failed to update transaksi status: {}", e)))
            }
        }
    }

    async fn cancel_transaksi(
        &self,
        request: Request<TransaksiId>,
    ) -> Result<Response<TransaksiResponse>, Status> {
        let id = request.into_inner().id;
        debug!("Received CancelTransaksi request for id: {}", id);
        
        let uuid = match Uuid::parse_str(&id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(Status::invalid_argument("Invalid UUID format")),
        };
        
        match self.service.cancel_transaksi(uuid).await {
            Ok(transaksi) => {
                let response = self.transaksi_to_response(&transaksi);
                Ok(Response::new(response))
            },
            Err(e) => {
                error!("Failed to cancel transaksi: {}", e);
                Err(Status::internal(format!("Failed to cancel transaksi: {}", e)))
            }
        }
    }

    type WatchTransaksiStream = Pin<Box<dyn Stream<Item = Result<TransaksiEvent, Status>> + Send + 'static>>;

    async fn watch_transaksi(
        &self,
        request: Request<TransaksiId>,
    ) -> Result<Response<Self::WatchTransaksiStream>, Status> {
        let id = request.into_inner().id;
        debug!("Received WatchTransaksi request for id: {}", id);
        
        let uuid = match Uuid::parse_str(&id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(Status::invalid_argument("Invalid UUID format")),
        };
        
        let (tx, rx) = mpsc::channel(128);
        
        let observer = TransaksiObserver::new(tx);
        
        if let Err(e) = self.service.register_observer(uuid, Box::new(observer)).await {
            error!("Failed to register observer: {}", e);
            return Err(Status::internal("Failed to watch transaksi"));
        }
        
        let output_stream = tokio_stream::wrappers::ReceiverStream::new(rx)
            .map(Ok::<_, Status>);
        
        Ok(Response::new(Box::pin(output_stream) as Self::WatchTransaksiStream))
    }
}

struct TransaksiObserver {
    sender: mpsc::Sender<TransaksiEvent>,
}

impl TransaksiObserver {
    fn new(sender: mpsc::Sender<TransaksiEvent>) -> Self {
        Self { sender }
    }
    
    async fn send_event(&self, event_type: transaksi_event::EventType, transaksi: Transaksi) {
        let service_impl = TransaksiServiceImpl::new(Arc::new(TransaksiService::default()));
        let response = service_impl.transaksi_to_response(&transaksi);
        
        let event = TransaksiEvent {
            event_type: event_type as i32,
            transaksi: Some(response),
        };
        
        if let Err(e) = self.sender.send(event).await {
            error!("Failed to send event through channel: {}", e);
        }
    }
}

#[tonic::async_trait]
impl Observer<Transaksi> for TransaksiObserver {
    async fn on_created(&self, transaksi: Transaksi) {
        self.send_event(transaksi_event::EventType::Created, transaksi).await;
    }
    
    async fn on_updated(&self, transaksi: Transaksi) {
        self.send_event(transaksi_event::EventType::Updated, transaksi).await;
    }
    
    async fn on_status_changed(&self, transaksi: Transaksi) {
        self.send_event(transaksi_event::EventType::StatusChanged, transaksi).await;
    }
    
    async fn on_deleted(&self, transaksi: Transaksi) {
        self.send_event(transaksi_event::EventType::Deleted, transaksi).await;
    }
}