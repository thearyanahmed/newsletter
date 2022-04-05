use actix_web::{HttpResponse, web, ResponseError};
use sqlx::{PgPool, Transaction, Postgres};
use chrono::Utc;
use uuid::Uuid;
use crate::{domain::{NewSubscriber, SubscriberName, SubscriberEmail}, email_client::{EmailClient}};
use std::convert::{TryFrom, TryInto};
use crate::startup::ApplicationBaseUrl;
use rand::distributions::Alphanumeric;
use rand::{thread_rng,Rng};
use std::fmt::Formatter;
use actix_web::http::StatusCode;
use anyhow::Context;

pub struct StoreTokenError(sqlx::Error);

impl ResponseError for StoreTokenError {}

// #[derive(thiserror::Error)]
// pub enum SubscribeError {
//     #[error("{0}")]
//     ValidationError(String),
//     #[error("failed to store token")]
//     StoreTokenError(StoreTokenError),
//     #[error("failed to send email")]
//     SendEmailError(reqwest::Error),
//     #[error("failed to acquire db pool")]
//     PoolError(sqlx::Error),
//     #[error("failed to insert subscriber")]
//     InsertSubscribeError(sqlx::Error),
//     #[error("failed to commit")]
//     TransactionCommitError(sqlx::Error),
// }

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    ServerSideError(#[from] anyhow::Error),

    // #[error("failed to insert new subscriber in the database")]
    // InsertSubscriberError(#[source] sqlx::Error)
}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self,f)
    }
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self,f)
    }
}

// impl From <reqwest::Error> for SubscribeError {
//     fn from(e: reqwest::Error) -> Self {
//         // Self::SendEmailError(e)
//         Self::ServerSideError(e)
//     }
// }
//
// impl From<StoreTokenError> for SubscribeError {
//     fn from(e: StoreTokenError) -> Self {
//         // Self::StoreTokenError(e)
//         Self::ServerSideError(e)
//     }
// }

impl From<String> for SubscribeError {
    fn from(s: String) -> Self {
        Self::ValidationError(s)
    }
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SubscribeError::ServerSideError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "a database error was encountered while trying to store subscription token"
        )
    }
}

fn error_chain_fmt(e: &impl std::error::Error, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f,"{}\n",e)?;

    let mut current = e.source();

    while let Some(cause) = current {
        writeln!(f, "caused by:\n\t{}",cause)?;
        current = cause.source();
    }

    Ok(())
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
) -> Result<HttpResponse, SubscribeError> {

    let new_subscriber = form.0.try_into()
        .map_err(SubscribeError::ValidationError)?;

    let mut transaction = pool.begin()
        .await
        .context("failed to acquire a postgres connection from the pool")?;

    // create a subscriber record
    let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber)
        .await
        .context("failed to insert subscriber id")?;

    let subscription_token = generate_subscription_token();

    store_token(&mut transaction, subscriber_id, &subscription_token)
        .await
        .context("failed to store token")?;

    transaction.commit()
        .await
        .context("failed to commit")?;

    send_confirmation_email(&email_client, new_subscriber, &base_url.0, &subscription_token)
        .await
        .context("failed to send confirmation email")?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "store subscription token in db",
    skip(subscription_token,transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str
) -> Result<(),StoreTokenError> {
    sqlx::query!(
            r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id) VALUES ($1, $2)"#,
            subscription_token, subscriber_id
        )
        .execute(transaction)
        .await
        .map_err(|e| StoreTokenError(e))?;

    Ok(())
}



#[tracing::instrument(
    name = "send a confirmation email to a new subscriber",
    skip(email_client,new_subscriber,base_url,subscription_token)
)]
async fn send_confirmation_email(email_client: &EmailClient, new_subscriber: NewSubscriber, base_url: &str,subscription_token: &str) -> Result<(), reqwest::Error> {
    // as I don't have post-map api
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
    skip(transaction,new_subscriber)
)]
pub async fn insert_subscriber(transaction: &mut Transaction<'_,Postgres>, new_subscriber: &NewSubscriber) -> Result<Uuid,sqlx::Error> {
    let uuid = Uuid::new_v4();

    sqlx::query!(
            r#"
                INSERT INTO subscriptions (id, email, name, subscribed_at, status)
                VALUES ($1,$2,$3,$4, 'pending_confirmation')
            "#,
            uuid, new_subscriber.email.as_ref(), new_subscriber.name.as_ref(), Utc::now()
        )
        .execute(transaction)
        .await?;

    Ok(uuid)
}