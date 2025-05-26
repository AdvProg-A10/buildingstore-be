use crate::manajemen_pembayaran::model::payment::PaymentMethod;
use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;
use crate::manajemen_pembayaran::patterns::state::{PaymentState, PaidState, InstallmentState};

pub struct PaymentStateFactory;
impl PaymentStateFactory {
    pub fn create(status: &PaymentStatus) -> Box<dyn PaymentState> {
        match status {
            PaymentStatus::Paid => Box::new(PaidState),
            PaymentStatus::Installment => Box::new(InstallmentState),
        }
    }
}
