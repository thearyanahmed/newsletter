#[tokio::test]
async fn health_check_works() {
    spawn_app().await.expect("failed to spawn app")

    let client = reqwest::Client::new();

    let response = client.get("http://localhost:8000/health_check")
        .send()
        .await
        .expect("failed to execute test");

    assert!(response.status().is_success());
    assert!(Some(0),response.content_length());
}

async fn spawn_app() -> std::io::Result<()> {
    newsletter::run().await
}