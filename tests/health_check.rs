use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check",&address))
        .send()
        .await
        .expect("failed to execute test");

    assert!(response.status().is_success());
    assert_eq!(Some(0),response.content_length());
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind to random port");

    let port = listener.local_addr().unwrap().port();

    let server = newsletter::run(listener).expect("failed to bind address.");

    let _ = tokio::spawn(server);

    return format!("http://127.0.0.1:{}",port);
}