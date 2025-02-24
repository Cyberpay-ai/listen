use std::sync::Arc;

use crate::engine::{
    order::{swap_order_to_transaction, SwapOrder, SwapOrderTransaction},
    Engine, EngineError,
};
use blockhash_cache::{inject_blockhash_into_encoded_tx, BLOCKHASH_CACHE};
use privy::{tx::PrivyTransaction, Privy};

impl Engine {
    pub async fn execute_order(
        &self,
        order: &SwapOrder,
        user_id: &str,
        wallet_address: &str,
        pubkey: &str,
    ) -> Result<String, EngineError> {
        // Execute transaction first
        let address = match order.is_evm() {
            true => wallet_address,
            false => pubkey,
        };
        let mut privy_transaction = PrivyTransaction {
            user_id: user_id.to_string(),
            address: address.to_string(),
            from_chain_caip2: order.from_chain_caip2.clone(),
            to_chain_caip2: order.to_chain_caip2.clone(),
            evm_transaction: None,
            solana_transaction: None,
        };

        let transaction_result =
            match swap_order_to_transaction(order, &lifi::LiFi::new(None), wallet_address, pubkey)
                .await
                .map_err(EngineError::SwapOrderError)?
            {
                SwapOrderTransaction::Evm(transaction) => {
                    let spender_address = transaction["to"].as_str().unwrap();
                    ensure_approvals(
                        spender_address,
                        order,
                        &privy_transaction,
                        self.privy.clone(),
                    )
                    .await?;
                    privy_transaction.evm_transaction = Some(transaction);
                    self.privy.execute_transaction(privy_transaction).await
                }
                SwapOrderTransaction::Solana(transaction) => {
                    let latest_blockhash = BLOCKHASH_CACHE
                        .get_blockhash()
                        .await
                        .map_err(EngineError::BlockhashCacheError)?;
                    let fresh_blockhash_tx = inject_blockhash_into_encoded_tx(
                        &transaction,
                        &latest_blockhash.to_string(),
                    )
                    .map_err(EngineError::InjectBlockhashError)?;
                    privy_transaction.solana_transaction = Some(fresh_blockhash_tx);
                    self.privy.execute_transaction(privy_transaction).await
                }
            };

        transaction_result.map_err(EngineError::TransactionError)
    }
}

pub async fn ensure_approvals(
    spender_address: &str,
    order: &SwapOrder,
    privy_transaction: &PrivyTransaction,
    privy: Arc<Privy>,
) -> Result<(), EngineError> {
    let allowance = approvals::get_allowance(
        &order.input_token,
        &privy_transaction.address,
        spender_address,
        approvals::caip2_to_chain_id(&order.from_chain_caip2).unwrap(),
    )
    .await
    .map_err(EngineError::ApprovalsError)?;
    if allowance < order.amount.parse::<u128>().unwrap() {
        let approval_transaction = approvals::create_approval_transaction(
            &order.input_token,
            spender_address,
            &privy_transaction.address,
            approvals::caip2_to_chain_id(&order.from_chain_caip2).unwrap(),
        )
        .await
        .map_err(EngineError::ApprovalsError)?;
        let mut approval_privy_tx = privy_transaction.clone();
        approval_privy_tx.evm_transaction = Some(approval_transaction);
        privy
            .execute_transaction(approval_privy_tx)
            .await
            .map_err(EngineError::TransactionError)?;
    }
    Ok(())
}
