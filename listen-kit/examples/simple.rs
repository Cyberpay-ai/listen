#[cfg(feature = "solana")]
use {
    anyhow::Result,
    listen_kit::reasoning_loop::ReasoningLoop,
    listen_kit::signer::solana::LocalSolanaSigner,
    listen_kit::signer::SignerContext,
    listen_kit::solana::tools::GetPortfolio,
    listen_kit::solana::util::env,
    rig::{message::Message, message::UserContent, OneOrMany},
    std::sync::Arc,
};
use rig::providers::anthropic::ClientBuilder;

#[cfg(feature = "solana")]
#[tokio::main]
async fn main() -> Result<()> {
    use rig::completion::Chat;

    let signer = LocalSolanaSigner::new(env("SOLANA_PRIVATE_KEY"));

    SignerContext::with_signer(Arc::new(signer), async {
        let api_key =  std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY not set");
        let base_url = std::env::var("ANTHROPIC_BASE_URL").expect("ANTHROPIC_BASE_URL not set");
        let agent = ClientBuilder::new(&api_key)
            .base_url(&base_url)
            .build()
            .agent("claude-3-sonnet-20240229")
            .preamble("you are a portfolio checker, if you do wanna call a tool, outline the reasoning why that tool")
            .max_tokens(1024)
            .tool(GetPortfolio)
            .build();
        
        let res = agent.chat("hello", vec![]).await?;
        println!("hello response: {:?}", res);

        let agent = ReasoningLoop::new(Arc::new(agent));

        let msg = agent.stream(
            vec![
                Message::User {
                    content: OneOrMany::one(
                        UserContent::text("whats the portfolio looking like?".to_string())
                    ),
                }
            ], None).await?;

        println!("msg: {:?}", msg);
        
        // if let Err(e) = msg {
        //     let _ = tx
        //         .send(sse::Event::Data(sse::Data::new(
        //             serde_json::to_string(&StreamResponse::Error(
        //                 e.to_string(),
        //             ))
        //             .unwrap(),
        //         )))
        //         .await;
        // }
        Ok(())
    })
    .await
}

#[cfg(not(feature = "solana"))]
fn main() {
    println!("enable the solana feature to run this example");
}
