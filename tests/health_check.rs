#[tokio::test]
async fn health_check_works() {
    spawn_app();

    let client = reqwest::Client::new();

    let response = client.get("http://localhost:8000/health_check")
        .send()
        .await
        .expect("failed to execute test");

    assert!(response.status().is_success());
    assert_eq!(Some(0),response.content_length());
}

fn spawn_app() {
    let server = newsletter::run().expect("failed to bind address.");

    let _ = tokio::spawn(server);
}