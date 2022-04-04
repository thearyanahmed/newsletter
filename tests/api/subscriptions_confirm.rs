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
    let body : serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    let get_link = |s: &str| {
        let links : Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkFinder::Url)
            .collect_vec();

        assert_eq!(links.len(),1);

        links[0].as_str().to_owned()
    };

    let raw_confirmation_link = &get_link(&body["html_body"].as_str().wrap());
    let confirmation_link = Url::parse(raw_confirmation_link).unwrap();

    assert_eq!(confirmation_link.host_str().unwrap(),"127.0.0.1");

    let response = reqwest::get(confirmation_link)
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(),200);

}