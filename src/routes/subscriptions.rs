use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;
use crate::{domain::{NewSubscriber, SubscriberName, SubscriberEmail}, email_client::{EmailClient}};
use std::convert::TryInto;

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

#[tracing::instrument(
    name = "adding a new subscriber",
    skip(form,pool,email_client),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>,email_client: web::Data<EmailClient>) -> HttpResponse {
    let new_subscriber = match form.0.try_into() {
        Ok(sub) => sub,
        Err(_) => return HttpResponse::BadRequest().finish() 
    };

    if insert_subscriber(&pool,&new_subscriber).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    if email_client.send_email(new_subscriber.email, "Welcome!", "Welcome to our newsletter!", "Welcome to our newsletter!")
        .await
        .is_err() {
            return HttpResponse::InternalServerError().finish();
        }

    HttpResponse::Ok().finish()
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
pub async fn insert_subscriber(pool: &PgPool, new_subscriber: &NewSubscriber) -> Result<(),sqlx::Error> {
    sqlx::query!(
            r#"
                INSERT INTO subscriptions (id, email, name, subscribed_at, status)
                VALUES ($1,$2,$3,$4, 'confirmed')
            "#,
            Uuid::new_v4(), new_subscriber.email.as_ref(), new_subscriber.name.as_ref(), Utc::now()
        )
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("failed to execute query: {:?}",e);
            e
        })?;

    Ok(())
}