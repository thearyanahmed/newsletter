use reqwest::Client;
use secrecy::{Secret, ExposeSecret};
use crate::domain::SubscriberEmail;

#[derive(Clone)]
pub struct EmailClient {
    sender: SubscriberEmail,
    http_client: Client,
    base_url: String,   
    authorization_token : Secret<String>,
}

#[derive(serde::Serialize)]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail, authorization_token: Secret<String>) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap();

        Self {
            http_client,
            base_url,
            sender,
            authorization_token
        }
    }

    pub async fn send_email(&self, recipient: SubscriberEmail, subject: &str, html_body: &str, text_body: &str) -> Result<(), reqwest::Error> {
        let url = format!("{}/email",self.base_url);

        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body,
            text_body,
        };

        self
            .http_client
            .post(&url)
            .header("X-POSTMARK-SERVER-TOKEN", self.authorization_token.expose_secret())
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use secrecy::Secret;
    use claim::{assert_ok,assert_err};
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph,Sentence};
    use fake::{Fake,Faker};
    use wiremock::matchers::{header,header_exists,path};
    use wiremock::{Request,Mock,MockServer,ResponseTemplate};
    
    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let r : Result<serde_json::Value, _> = serde_json::from_slice(&request.body);

            if let Ok(body) = r {
                body.get("from").is_some() 
                    && body.get("to").is_some() 
                    && body.get("subject").is_some() 
                    && body.get("html_body").is_some() 
                    && body.get("text_body").is_some() 
                 
            } else {
                false
            }
        }
    }

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Paragraph(1..10).fake()
    }

    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server  = MockServer::start().await;
        let sender       = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(),sender, Secret::new(Faker.fake()));

        Mock::given(header_exists("X-POSTMARK-SERVER-TOKEN"))
            .and(header("Content-Type","application/json"))
            .and(path("/email"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let recipient       = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        let outcome = email_client
            .send_email(recipient, &subject, &content, &content)
            .await;

        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        let mock_server  = MockServer::start().await;
        let sender       = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(),sender, Secret::new(Faker.fake()));

        Mock::given(header_exists("X-POSTMARK-SERVER-TOKEN"))
            .and(header("Content-Type","application/json"))
            .and(path("/email"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let recipient       = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();  
        
        let outcome = email_client
            .send_email(recipient, &subject, &content, &content)
            .await;

        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        let mock_server  = MockServer::start().await;
        let sender       = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(),sender, Secret::new(Faker.fake()));

        let response = ResponseTemplate::new(200)
            .set_delay(std::time::Duration::from_secs(180));

        Mock::given(header_exists("X-POSTMARK-SERVER-TOKEN"))
            .and(header("Content-Type","application/json"))
            .and(path("/email"))
            .and(SendEmailBodyMatcher)
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let recipient       = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();  
        
        let outcome = email_client
            .send_email(recipient, &subject, &content, &content)
            .await;

        assert_err!(outcome);
    }
}