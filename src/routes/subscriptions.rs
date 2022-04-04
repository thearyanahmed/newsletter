use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;
use crate::{domain::{NewSubscriber, SubscriberName, SubscriberEmail}, email_client::{EmailClient}};
use std::convert::TryInto;
use crate::startup::ApplicationBaseUrl;
use rand::distributions::Alphanumeric;
use rand::{thread_rng,Rng};

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;

        Ok(Self{ email, name })
    }
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();

    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "adding a new subscriber",
    skip(form,pool,email_client,base_url),
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>
) -> HttpResponse {
    let new_subscriber = match form.0.try_into() {
        Ok(sub) => sub,
        Err(_) => return HttpResponse::BadRequest().finish() 
    };

    let subscriber_id = match insert_subscriber(&pool,&new_subscriber).await {
        Ok(sub_id) => sub_id,
        Err(_) => return HttpResponse::InternalServerError().finish()
    };

    let subscription_token = generate_subscription_token();

    if store_token(&pool, subscriber_id, &subscription_token)
        .await
        .is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    if send_confirmation_email(&email_client,new_subscriber,&base_url.0,&subscription_token)
        .await
        .is_err() {
            return HttpResponse::InternalServerError().finish();
        }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "store subscription token in db",
    skip(subscription_token,pool)
)]
pub async fn store_token(pool: &PgPool, subscriber_id: Uuid, subscription_token: &str) -> Result<(),sqlx::Error> {
    sqlx::query!(
            r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id) VALUES ($1, $2)"#,
            subscription_token, subscriber_id
        )
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("failed to execute query: {:?}",e);
            e
        })?;

    Ok(())
}

#[tracing::instrument(
    name = "send a confirmation email to a new subscriber",
    skip(email_client,new_subscriber,base_url,subscription_token)
)]
async fn send_confirmation_email(email_client: &EmailClient, new_subscriber: NewSubscriber, base_url: &str,subscription_token: &str) -> Result<(), reqwest::Error> {
    // as I don't have postmap api
    let confirmation_link = format!("{}/subscriptions/confirm?subscription_token={}",base_url,subscription_token);

    println!("send_confirmation_email link => {}",confirmation_link);

    let html_body = format!(
        "Welcome to our newsletter! <br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );

    let text_body = format!(
        "Welcome to our newsletter!\n Visit {} to confirm your subscription.",
        confirmation_link
    );

    email_client
        .send_email(
            new_subscriber.email,
            "Welcome!",
            &html_body,
            &text_body
        )
        .await
}

pub fn parse_subscriber(form: FormData) -> Result<NewSubscriber, String> {
    let name = SubscriberName::parse(form.name)?;
    let email = SubscriberEmail::parse(form.email)?;

    Ok(NewSubscriber{ email, name })
}

#[tracing::instrument(
    name = "saving new subscriber details in the database",
    skip(pool,new_subscriber)
)]
pub async fn insert_subscriber(pool: &PgPool, new_subscriber: &NewSubscriber) -> Result<Uuid,sqlx::Error> {
    let uuid = Uuid::new_v4();

    sqlx::query!(
            r#"
                INSERT INTO subscriptions (id, email, name, subscribed_at, status)
                VALUES ($1,$2,$3,$4, 'pending_confirmation')
            "#,
            uuid, new_subscriber.email.as_ref(), new_subscriber.name.as_ref(), Utc::now()
        )
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("failed to execute query: {:?}",e);
            e
        })?;

    Ok(uuid)
}