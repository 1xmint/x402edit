#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().compact().init();
    tracing::info!(
        "worker started; live provider dispatch is disabled until policy and credentials are configured"
    );
    std::future::pending::<()>().await;
}
