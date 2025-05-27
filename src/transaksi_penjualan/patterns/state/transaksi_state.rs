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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_permissions() {
        let processing_state = MasihDiprosesState;
        assert!(processing_state.can_be_modified());
        assert!(processing_state.can_be_cancelled());
        assert!(processing_state.can_be_completed());
        assert!(processing_state.can_add_items());

        let completed_state = SelesaiState;
        assert!(!completed_state.can_be_modified());
        assert!(!completed_state.can_be_cancelled());
        assert!(!completed_state.can_be_completed());
        assert!(!completed_state.can_add_items());

        let cancelled_state = DibatalkanState;
        assert!(!cancelled_state.can_be_modified());
        assert!(!cancelled_state.can_be_cancelled());
        assert!(!cancelled_state.can_be_completed());
        assert!(!cancelled_state.can_add_items());
    }

    #[test]
    fn test_state_transitions() {
        let processing_state = MasihDiprosesState;

        let completed = processing_state.next_state(StateAction::Complete).unwrap();
        assert_eq!(completed.status(), StatusTransaksi::Selesai);

        let cancelled = processing_state.next_state(StateAction::Cancel).unwrap();
        assert_eq!(cancelled.status(), StatusTransaksi::Dibatalkan);

        let result = processing_state.next_state(StateAction::Reopen);
        assert!(result.is_err());
    }

    #[test]
    fn test_state_factory() {
        let masih_diproses = TransaksiStateFactory::create_state(&StatusTransaksi::MasihDiproses);
        assert!(masih_diproses.can_be_modified());

        let selesai = TransaksiStateFactory::create_state(&StatusTransaksi::Selesai);
        assert!(!selesai.can_be_modified());

        let dibatalkan = TransaksiStateFactory::create_state(&StatusTransaksi::Dibatalkan);
        assert!(!dibatalkan.can_be_modified());
    }

    #[test]
    fn test_state_workflow() {
        let mut current_state = TransaksiStateFactory::create_state(&StatusTransaksi::MasihDiproses);
        assert!(current_state.can_be_modified());

        current_state = current_state.next_state(StateAction::Complete).unwrap();
        assert_eq!(current_state.status(), StatusTransaksi::Selesai);
        assert!(!current_state.can_be_modified());

        current_state = current_state.next_state(StateAction::Reopen).unwrap();
        assert_eq!(current_state.status(), StatusTransaksi::MasihDiproses);
        assert!(current_state.can_be_modified());

        current_state = current_state.next_state(StateAction::Cancel).unwrap();
        assert_eq!(current_state.status(), StatusTransaksi::Dibatalkan);
        assert!(!current_state.can_be_modified());
    }

    #[test]
    fn test_allowed_actions() {
        let processing_state = MasihDiprosesState;
        let actions = processing_state.get_allowed_actions();
        assert!(actions.contains(&"complete".to_string()));
        assert!(actions.contains(&"cancel".to_string()));
        assert!(actions.contains(&"add_item".to_string()));

        let completed_state = SelesaiState;
        let actions = completed_state.get_allowed_actions();
        assert!(actions.contains(&"print_receipt".to_string()));
        assert!(actions.contains(&"view_details".to_string()));
        assert!(!actions.contains(&"complete".to_string()));
    }
}