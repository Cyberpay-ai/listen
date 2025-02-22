#[cfg(feature = "http")]
use {
    listen_kit::http::server::run_server,
    privy::{config::PrivyConfig, Privy},
};

#[cfg(feature = "http")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let privy_client =
        Privy::new(PrivyConfig::from_env().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e)
        })?);

    print!("Starting server...");

    let omni_agent =
        listen_kit::cross_chain::agent::create_cross_chain_agent()
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    
    print!("omni_agent...");
    // Create agents based on enabled features
    #[cfg(feature = "solana")]
    let solana_agent = listen_kit::solana::agent::create_solana_agent()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    #[cfg(feature = "evm")]
    let evm_agent = listen_kit::evm::agent::create_evm_agent()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Run server with appropriate agents based on features
    #[cfg(all(feature = "solana", feature = "evm"))]
    return run_server(solana_agent, evm_agent, privy_client, omni_agent)
        .await;

    #[cfg(all(feature = "solana", not(feature = "evm")))]
    return run_server(solana_agent, privy_client, omni_agent).await;

    #[cfg(all(feature = "evm", not(feature = "solana")))]
    return run_server(evm_agent, privy_client, omni_agent).await;

    #[cfg(not(any(feature = "solana", feature = "evm")))]
    return run_server(privy_client, omni_agent).await;
}

#[cfg(not(feature = "http"))]
fn main() {
    println!("This binary requires the 'http' feature");
}
