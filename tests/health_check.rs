use std::net::TcpListener;
use newsletter::startup::run;

// Spawns an instance of the app.
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind to random port");

    let port = listener.local_addr().unwrap().port();

    let server = run(listener).expect("failed to bind address.");

    let _ = tokio::spawn(server);

    return format!("http://127.0.0.1:{}",port);
}

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

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let addr = spawn_app();
    let client = reqwest::Client::new();

    let form_body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions",&addr))
        .header("Content-Type","application/x-www-form-urlencoded")
        .body(form_body)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(200,response.status().as_u16());
}
#[tokio::test]
async fn subscribe_returns_a_400_for_invalid_form_data() {
    let addr = spawn_app();
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin","missing the email"),
        ("email=ursula_le_guin%40gmail.com","missing the name"),
        ("","missing both"),
    ];

    for (form_body, error) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions",&addr))
            .header("Content-Type","application/x-www-form-urlencoded")
            .body(form_body)
            .send()
            .await
            .expect("failed to execute request");

        assert_eq!(400,response.status().as_u16(),"the api did not fail with 400 bad request when the payload was {}",error);
    }
}

