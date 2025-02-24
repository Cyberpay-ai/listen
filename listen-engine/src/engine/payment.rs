use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use privy::caip2::Caip2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentOrder {
    pub input_token: String,
    pub output_token: String,
    pub amount: String,
    pub from_chain_caip2: String,
    pub to_chain_caip2: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PaymentOrderError {
    #[error("Invalid CAIP2")]
    InvalidCaip2,

    #[error("LiFi error: {0}")]
    LiFiError(lifi::LiFiError),

    #[error("No transaction request")]
    NoTransactionRequest,

    #[error("Serialize error: {0}")]
    SerializeError(anyhow::Error),
}

pub fn is_solana(caip2: &str) -> bool {
    caip2.starts_with("solana:")
}

pub fn is_evm(caip2: &str) -> bool {
    caip2.starts_with("eip155:")
}

impl PaymentOrder {
    pub fn is_evm(&self) -> bool {
        is_evm(&self.from_chain_caip2)
    }
}

// Map of CAIP2 identifiers to LiFi chain IDs
static CHAIN_ID_MAP: Lazy<HashMap<&'static str, u64>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // Solana special case
    m.insert(Caip2::SOLANA, 1151111081099710);
    // EVM chains
    m.insert(Caip2::ETHEREUM, 1);
    m.insert(Caip2::BSC, 56);
    m.insert(Caip2::ARBITRUM, 42161);
    m.insert(Caip2::BASE, 8453);
    m.insert(Caip2::BLAST, 81457);
    m.insert(Caip2::AVALANCHE, 43114);
    m.insert(Caip2::POLYGON, 137);
    m.insert(Caip2::SCROLL, 534352);
    m.insert(Caip2::OPTIMISM, 10);
    m.insert(Caip2::LINEA, 59144);
    m.insert(Caip2::GNOSIS, 100);
    m.insert(Caip2::FANTOM, 250);
    m.insert(Caip2::MOONRIVER, 1285);
    m.insert(Caip2::MOONBEAM, 1284);
    m.insert(Caip2::BOBA, 288);
    m.insert(Caip2::MODE, 34443);
    m.insert(Caip2::METIS, 1088);
    m.insert(Caip2::LISK, 1135);
    m.insert(Caip2::AURORA, 1313161554);
    m.insert(Caip2::SEI, 1329);
    m.insert(Caip2::IMMUTABLE, 13371);
    m.insert(Caip2::GRAVITY, 1625);
    m.insert(Caip2::TAIKO, 167000);
    m.insert(Caip2::CRONOS, 25);
    m.insert(Caip2::FRAXTAL, 252);
    m.insert(Caip2::ABSTRACT, 2741);
    m.insert(Caip2::CELO, 42220);
    m.insert(Caip2::WORLD, 480);
    m.insert(Caip2::MANTLE, 5000);
    m.insert(Caip2::BERACHAIN, 80094);
    m
});

fn caip2_to_chain_id(caip2: &str) -> Option<u64> {
    CHAIN_ID_MAP.get(caip2).copied()
}

pub enum PaymentOrderTransaction {
    Evm(serde_json::Value),
    Solana(String),
}

// pub async fn swap_order_to_transaction(
//     order: &PaymentOrder,
//     lifi: &lifi::LiFi,
//     wallet_address: &str, // evm output
//     pubkey: &str,         // solana output
// ) -> Result<PaymentOrderTransaction, PaymentOrderError> {
//     let from_chain_id =
//         caip2_to_chain_id(&order.from_chain_caip2).ok_or(PaymentOrderError::InvalidCaip2)?;
//     let to_chain_id =
//         caip2_to_chain_id(&order.to_chain_caip2).ok_or(PaymentOrderError::InvalidCaip2)?;

//     let from_address = if is_evm(&order.from_chain_caip2) {
//         wallet_address
//     } else {
//         pubkey
//     };

//     let to_address = if is_evm(&order.to_chain_caip2) {
//         wallet_address
//     } else {
//         pubkey
//     };

//     let quote = lifi
//         .get_quote(
//             &from_chain_id.to_string(),
//             &to_chain_id.to_string(),
//             &order.input_token,
//             &order.output_token,
//             from_address,
//             to_address,
//             &order.amount,
//         )
//         .await
//         .map_err(PaymentOrderError::LiFiError)?;

//     match quote.transaction_request {
//         Some(transaction_request) => {
//             if transaction_request.is_solana() {
//                 Ok(PaymentOrderTransaction::Solana(transaction_request.data))
//             } else {
//                 Ok(PaymentOrderTransaction::Evm(
//                     transaction_request
//                         .to_json_rpc()
//                         .map_err(PaymentOrderError::SerializeError)?,
//                 ))
//             }
//         }
//         None => Err(PaymentOrderError::NoTransactionRequest),
//     }
// }
