pub mod json;

use crate::auction::Side;
use crate::vm::Program;
use crate::{Price, ProductId};

pub trait WireProtocol {
    type Error: std::error::Error;
    fn try_client_directive_from_bytes(bytes: &[u8]) -> Result<ClientDirective, Self::Error>;
    fn try_client_directive_to_bytes(
        client_directive: &ClientDirective,
    ) -> Result<Vec<u8>, Self::Error>;
    fn try_client_notification_from_bytes(bytes: &[u8]) -> Result<ClientNotification, Self::Error>;
    fn try_client_notification_to_bytes(
        client_notification: &ClientNotification,
    ) -> Result<Vec<u8>, Self::Error>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientDirective {
    Join {},
    Leave {},
    UpdateParameter {
        product_id: ProductId,
        param_idx: u64,
        value: i64,
    },
    SubmitProgram {
        product_id: ProductId,
        program: Program,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientNotification {
    Trade {
        product_id: ProductId,
        side: Side,
        price: Price,
        quantity: u64,
    },
}
