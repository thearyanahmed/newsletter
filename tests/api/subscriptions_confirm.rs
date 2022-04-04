use crate::helpers::spawn_app;
use wiremock::{Mock, ResponseTemplate};
use wiremock::matchers::{path, method};
use reqwest::Url;

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_400() {
    let app = spawn_app().await;

    let response = reqwest::get(
        &format!("{}/subscriptions/confirm",app.address)
    ).await.unwrap();

    assert_eq!(response.status().as_u16(),400)
}

#[tokio::test]
async fn confirmation_link_url_returns_a_200_if_called() {
    let app = spawn_app().await;
    let body = "name=aryan&email=aryan@gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscription(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    let response = reqwest::get(confirmation_links.html)
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(),200);
}