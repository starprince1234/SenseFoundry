#[tokio::main]
async fn main() {
    let app = api_server::app();
    let _ = app;
}
