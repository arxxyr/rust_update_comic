// mod.rs
use system_shutdown;
use tokio::time::{sleep, Duration};

pub async fn shutdown(delay: Option<u64>) -> Result<(), Box<dyn std::error::Error>> {
    let delay_secs = delay.unwrap_or(30);
    println!("Shutting down the system in {} seconds...", delay_secs);

    sleep(Duration::from_secs(delay_secs)).await;

    system_shutdown::shutdown()?;
    Ok(())
}
