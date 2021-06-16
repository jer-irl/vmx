use crate::auction::{Side, Trade};
use crate::server::OutgoingMessage;
use crate::vm::Program;
use crate::ProductId;

pub enum ClientDirective {
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

impl From<&[u8]> for ClientDirective {
    fn from(_bytes: &[u8]) -> Self {
        panic!("Unimplemented");
    }
}

pub enum ClientNotification {
    Trade {
        product_id: ProductId,
        side: Side,
        quantity: u64,
    },
}

impl From<&Trade> for ClientNotification {
    fn from(_t: &Trade) -> Self {
        panic!("Unimplemented");
    }
}

impl From<ClientNotification> for OutgoingMessage {
    fn from(_notification: ClientNotification) -> Self {
        panic!("Unimplemented");
    }
}
