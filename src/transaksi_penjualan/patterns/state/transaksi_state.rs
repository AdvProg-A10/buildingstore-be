// transaksi/patterns/transaksi_state.rs

use crate::transaksi_penjualan::enums::status_transaksi::StatusTransaksi;

pub trait TransaksiState: Send + Sync {
    fn can_be_modified(&self) -> bool;
    fn can_be_cancelled(&self) -> bool;
    fn can_be_completed(&self) -> bool;
    fn can_add_items(&self) -> bool;
    fn can_update_items(&self) -> bool;
    fn can_delete_items(&self) -> bool;
    fn next_state(&self, action: StateAction) -> Result<Box<dyn TransaksiState>, String>;
    fn status(&self) -> StatusTransaksi;
    fn get_allowed_actions(&self) -> Vec<String>;
}

#[derive(Debug, Clone)]
pub enum StateAction {
    Complete,
    Cancel,
    Reopen,
}

// State: Masih Diproses
#[derive(Debug, Clone)]
pub struct MasihDiprosesState;

impl TransaksiState for MasihDiprosesState {
    fn can_be_modified(&self) -> bool { true }
    fn can_be_cancelled(&self) -> bool { true }
    fn can_be_completed(&self) -> bool { true }
    fn can_add_items(&self) -> bool { true }
    fn can_update_items(&self) -> bool { true }
    fn can_delete_items(&self) -> bool { true }
    
    fn next_state(&self, action: StateAction) -> Result<Box<dyn TransaksiState>, String> {
        match action {
            StateAction::Complete => Ok(Box::new(SelesaiState)),
            StateAction::Cancel => Ok(Box::new(DibatalkanState)),
            StateAction::Reopen => Err("Transaksi sudah dalam status diproses".to_string()),
        }
    }
    
    fn status(&self) -> StatusTransaksi { StatusTransaksi::MasihDiproses }
    
    fn get_allowed_actions(&self) -> Vec<String> {
        vec!["complete".to_string(), "cancel".to_string(), "add_item".to_string(), "update_item".to_string()]
    }
}

// State: Selesai
#[derive(Debug, Clone)]
pub struct SelesaiState;

impl TransaksiState for SelesaiState {
    fn can_be_modified(&self) -> bool { false }
    fn can_be_cancelled(&self) -> bool { false }
    fn can_be_completed(&self) -> bool { false }
    fn can_add_items(&self) -> bool { false }
    fn can_update_items(&self) -> bool { false }
    fn can_delete_items(&self) -> bool { false }
    
    fn next_state(&self, action: StateAction) -> Result<Box<dyn TransaksiState>, String> {
        match action {
            StateAction::Reopen => Ok(Box::new(MasihDiprosesState)),
            _ => Err("Transaksi selesai tidak dapat diubah statusnya".to_string()),
        }
    }
    
    fn status(&self) -> StatusTransaksi { StatusTransaksi::Selesai }
    
    fn get_allowed_actions(&self) -> Vec<String> {
        vec!["print_receipt".to_string(), "view_details".to_string()]
    }
}

// State: Dibatalkan
#[derive(Debug, Clone)]
pub struct DibatalkanState;

impl TransaksiState for DibatalkanState {
    fn can_be_modified(&self) -> bool { false }
    fn can_be_cancelled(&self) -> bool { false }
    fn can_be_completed(&self) -> bool { false }
    fn can_add_items(&self) -> bool { false }
    fn can_update_items(&self) -> bool { false }
    fn can_delete_items(&self) -> bool { false }
    
    fn next_state(&self, action: StateAction) -> Result<Box<dyn TransaksiState>, String> {
        match action {
            StateAction::Reopen => Ok(Box::new(MasihDiprosesState)),
            _ => Err("Transaksi dibatalkan tidak dapat diubah statusnya".to_string()),
        }
    }
    
    fn status(&self) -> StatusTransaksi { StatusTransaksi::Dibatalkan }
    
    fn get_allowed_actions(&self) -> Vec<String> {
        vec!["view_details".to_string()]
    }
}

// Factory untuk membuat state
pub struct TransaksiStateFactory;

impl TransaksiStateFactory {
    pub fn create_state(status: &StatusTransaksi) -> Box<dyn TransaksiState> {
        match status {
            StatusTransaksi::MasihDiproses => Box::new(MasihDiprosesState),
            StatusTransaksi::Selesai => Box::new(SelesaiState),
            StatusTransaksi::Dibatalkan => Box::new(DibatalkanState),
        }
    }
}