use rocket::fairing::AdHoc;
use rocket::routes;
use sqlx::{Any, Pool};
use std::sync::Arc;

use crate::manajemen_supplier::repository::supplier_repository::SupplierRepository;
use crate::manajemen_supplier::service::supplier_notifier::SupplierNotifier;
use crate::manajemen_supplier::service::supplier_service::SupplierService;
use crate::manajemen_supplier::repository::supplier_transaction_repository::SupplierTransactionRepository;
use crate::manajemen_supplier::service::supplier_observer::SupplierObserver;

use crate::manajemen_supplier::repository::supplier_repository_impl::SupplierRepositoryImpl;
use crate::manajemen_supplier::service::supplier_dispatcher::SupplierDispatcher;
use crate::manajemen_supplier::service::supplier_service_impl::SupplierServiceImpl;
use crate::manajemen_supplier::repository::supplier_transaction_repository_impl::SupplierTransactionRepositoryImpl;
use crate::manajemen_supplier::service::supplier_transaction_logger::SupplierTransactionLogger;

pub mod supplier_controller;

pub fn route_stage() -> AdHoc {
    AdHoc::on_ignite("Supplier Module: Manage Dependencies & Init Routes", |rocket| async {
        let db_pool = match rocket.state::<Pool<Any>>() {
            Some(pool) => pool.clone(),
            None => {
                panic!("Critical dependency Pool<Any> not found for Supplier module. Halting startup.");
            }
        };

        let supplier_repository_instance: Arc<dyn SupplierRepository> =
            Arc::new(SupplierRepositoryImpl::new());
        let supplier_transaction_repository_instance: Arc<dyn SupplierTransactionRepository + Send + Sync> =
            Arc::new(SupplierTransactionRepositoryImpl::new());
        let supplier_dispatcher_instance = Arc::new(SupplierDispatcher::new());
        let supplier_service_instance: Arc<dyn SupplierService> =
            Arc::new(SupplierServiceImpl::new(
                supplier_repository_instance,
                supplier_transaction_repository_instance.clone(),
                supplier_dispatcher_instance.clone(),
            ));
        let transaction_logger_observer: Arc<dyn SupplierObserver + Send + Sync> =
            Arc::new(SupplierTransactionLogger::new(
                supplier_transaction_repository_instance.clone(),
                db_pool.clone(),
            ));

        supplier_dispatcher_instance.register(transaction_logger_observer);
        eprintln!("[SETUP DEBUG] SupplierTransactionLogger registered with SupplierDispatcher.");

        rocket
            .manage(supplier_service_instance)
            .manage(supplier_dispatcher_instance as Arc<dyn SupplierNotifier>)
            .mount("/api", routes![
                supplier_controller::save_supplier,
                supplier_controller::delete_supplier,
                supplier_controller::get_supplier,
                supplier_controller::update_supplier,
                supplier_controller::get_all_suppliers,
                supplier_controller::get_all_supplier_transactions
            ])
    })
}