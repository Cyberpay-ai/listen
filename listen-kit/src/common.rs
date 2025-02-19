use anyhow::{anyhow, Result};
use std::future::Future;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::signer::{SignerContext, TransactionSigner};

pub async fn wrap_unsafe<F, Fut, T>(f: F) -> Result<T>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    let (tx, mut rx) = mpsc::channel(1);

    tokio::spawn(async move {
        let result = f().await;
        let _ = tx.send(result).await;
    });

    rx.recv().await.ok_or_else(|| anyhow!("Channel closed"))?
}

pub async fn spawn_with_signer<F, Fut, T>(
    signer: Arc<dyn TransactionSigner>,
    f: F,
) -> tokio::task::JoinHandle<Result<T>>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    tokio::spawn(async move {
        SignerContext::with_signer(signer, async { f().await }).await
    })
}

use rig::agent::{Agent, AgentBuilder};
use rig::providers::anthropic::completion::CompletionModel as AnthropicCompletionModel;
use rig::providers::anthropic::ClientBuilder;

// pub fn claude_agent_builder() -> AgentBuilder<AnthropicCompletionModel> {
//     rig::providers::anthropic::Client::from_env()
//         .agent(rig::providers::anthropic::CLAUDE_3_5_SONNET)
// }
pub fn claude_agent_builder() -> AgentBuilder<AnthropicCompletionModel> {
    let api_key =  std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY not set");
    let base_url = std::env::var("ANTHROPIC_BASE_URL").expect("ANTHROPIC_BASE_URL not set");
    ClientBuilder::new(&api_key)
        .base_url(&base_url)
        .build()
        .agent("claude-3-sonnet-20240229")
}
pub async fn plain_agent() -> Result<Agent<AnthropicCompletionModel>> {
    Ok(claude_agent_builder()
        .preamble("be nice to the users")
        .max_tokens(1024)
        .build())
}

pub const PREAMBLE_COMMON: &str = "";
