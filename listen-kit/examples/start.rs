
use std::sync::Arc;
use listen_kit::solana::agent::create_solana_agent;
use listen_kit::signer::SignerContext;
use listen_kit::signer::solana::LocalSolanaSigner;
use rig::completion::Chat;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let private_key = std::env::var("SOLANA_PRIVATE_KEY")?;

    let signer = LocalSolanaSigner::new(private_key);

    SignerContext::with_signer(Arc::new(signer), async {
        let agent = create_solana_agent()
            .await?;
        
        let response = agent.chat("whats the portfolio looking like?", vec![]).await?;
        println!("{:?}", response);

        Ok(())
    })
    .await
}