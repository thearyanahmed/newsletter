use std::net::TcpListener;
use newsletter::startup::run;
use newsletter::configuration::{get_configuration, Settings};
use sqlx::{PgConnection,Connection};

// Spawns an instance of the app. It binds to a random port.
fn spawn_app() -> String {
    let config = get_configuration().expect("could not load config");

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

    let config = get_configuration().expect("failed to read config");

    let connection_string = config.database.connection_string();

    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("failed to connect to database");

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

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("failed to fetch saved subscriptions");

    assert_eq!(saved.email,"ursula_le_guin@gmail.com");
    assert_eq!(saved.name,"le guin");
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
