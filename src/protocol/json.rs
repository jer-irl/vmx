use serde::{Deserialize, Serialize};
use serde_json::{self, error as serde_error};

use super::{ClientDirective, ClientNotification, WireProtocol};
use crate::auction::Side;
use crate::{Price, ProductId};

pub struct JsonProtocol;

#[derive(Debug)]
pub enum Error {
    JsonDeserializeError,
    Other,
}

impl From<serde_error::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        match e.classify() {
            serde_error::Category::Data | serde_error::Category::Syntax => {
                Error::JsonDeserializeError
            }
            serde_error::Category::Io | serde_error::Category::Eof => Error::Other,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error {}

impl WireProtocol for JsonProtocol {
    type Error = Error;

    fn try_client_directive_from_bytes(bytes: &[u8]) -> Result<ClientDirective, Self::Error> {
        let directive =
            ClientDirective::from(&serde_json::from_slice::<JsonClientDirective>(bytes)?);
        Ok(directive)
    }

    fn try_client_directive_to_bytes(
        client_directive: &ClientDirective,
    ) -> Result<Vec<u8>, Self::Error> {
        let bytes = serde_json::to_vec(&JsonClientDirective::from(client_directive))?;
        Ok(bytes)
    }

    fn try_client_notification_from_bytes(bytes: &[u8]) -> Result<ClientNotification, Self::Error> {
        let notification =
            ClientNotification::from(&serde_json::from_slice::<JsonClientNotification>(bytes)?);
        Ok(notification)
    }

    fn try_client_notification_to_bytes(
        client_notification: &ClientNotification,
    ) -> Result<Vec<u8>, Self::Error> {
        let bytes = serde_json::to_vec(&JsonClientNotification::from(client_notification))?;
        Ok(bytes)
    }
}

#[derive(Serialize, Deserialize)]
enum JsonClientDirective {
    Join {},
    Leave {},
    UpdateParameter {
        product_id: u64,
        param_idx: u64,
        value: i64,
    },
    SubmitProgram {
        product_id: u64,
        program: Vec<u8>,
    },
}

impl From<&ClientDirective> for JsonClientDirective {
    fn from(directive: &ClientDirective) -> Self {
        match directive {
            ClientDirective::Join {} => JsonClientDirective::Join {},
            ClientDirective::Leave {} => JsonClientDirective::Leave {},
            ClientDirective::SubmitProgram {
                product_id,
                program,
            } => JsonClientDirective::SubmitProgram {
                product_id: product_id.0,
                program: Vec::from(&program.get_string()[..]),
            },
            ClientDirective::UpdateParameter {
                product_id,
                param_idx,
                value,
            } => JsonClientDirective::UpdateParameter {
                product_id: product_id.0,
                param_idx: *param_idx,
                value: *value,
            },
        }
    }
}

impl From<&JsonClientDirective> for ClientDirective {
    fn from(_: &JsonClientDirective) -> Self {
        todo!();
    }
}

#[derive(Serialize, Deserialize)]
enum JsonClientNotification {
    Trade {
        product_id: u64,
        side: String,
        price: u64,
        quantity: u64,
    },
}

impl From<&ClientNotification> for JsonClientNotification {
    fn from(n: &ClientNotification) -> Self {
        match n {
            ClientNotification::Trade {
                product_id,
                price,
                quantity,
                side,
            } => JsonClientNotification::Trade {
                product_id: product_id.0,
                price: price.0,
                quantity: *quantity,
                side: side.to_string(),
            },
        }
    }
}

impl From<&JsonClientNotification> for ClientNotification {
    fn from(n: &JsonClientNotification) -> Self {
        match n {
            JsonClientNotification::Trade {
                product_id,
                side,
                price,
                quantity,
            } => ClientNotification::Trade {
                product_id: ProductId(*product_id),
                side: match &side[..] {
                    "Bid" => Side::Bid,
                    "Offer" => Side::Offer,
                    _ => panic!(),
                },
                price: Price(*price),
                quantity: *quantity,
            },
        }
    }
}
