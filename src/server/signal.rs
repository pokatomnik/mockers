use std::pin::Pin;

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

pub fn make_signal() -> Pin<Box<dyn Future<Output = ()> + Send>> {
    Box::pin(shutdown_signal())
}
