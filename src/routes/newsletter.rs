use actix_web::{HttpResponse, ResponseError};
use actix_web::web;
use sqlx::PgPool;
use crate::routes::error_chain_fmt;
use actix_web::http::StatusCode;
use std::fmt::Formatter;
use actix_web::body::BoxBody;
use std::any::TypeId;
use crate::email_client::EmailClient;
use anyhow::Context;
use crate::domain::SubscriberEmail;

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String
}

struct ConfirmedSubscriber {
    email: SubscriberEmail
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    ServerSideError(#[from] anyhow::Error)
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self,f)
    }
}

impl ResponseError for PublishError {
    fn status_code(&self) -> StatusCode {
        match self {
            PublishError::ServerSideError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(name = "get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(pool: &PgPool)
    -> Result<Vec<Result<ConfirmedSubscriber,anyhow::Error>>,anyhow::Error> {
    struct Row {
        email: String,
    }

    let rows = sqlx::query_as!(
            Row,
            r#"SELECT email FROM subscriptions WHERE status = 'confirmed'"#
        )
        .fetch_all(pool)
        .await?;

    let confirmed_subscribers = rows
        .into_inter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber {email}),
            Err(e) => Err(anyhow::anyhow!(err))
        })
        .collect();

    Ok(confirmed_subscribers)
}

#[tracing::instrument(name = "publish newsletter",  skip(body,pool,email_client))]
pub async fn publish_newsletter(
    body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>
) -> Result<HttpResponse,PublishError> {
    let subscribers = get_confirmed_subscribers(&pool).await?;

    for subscriber in subscribers {
        email_client
            .send_email(
                subscriber.email,
                &body.title,
                &body.content.html,
                &body.content.text
            )
            .await
            .with_context(|| format!("failed to send newsletter issue to"))?;
    }

    Ok(HttpResponse::Ok().finish())
}

