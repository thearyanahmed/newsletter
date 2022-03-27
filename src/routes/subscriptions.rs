use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;
use unicode_segmentation::UnicodeSegmentation;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[tracing::instrument(
    name = "adding a new subscriber",
    skip(form,pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    if is_invalid_name(&form.name) {
        return HttpResponse::BadRequest().finish();
    }

    match insert_subscriber(&pool,&form).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish()
    }
}

pub fn is_invalid_name(s: &str) -> bool {
    let is_empty_or_whitespace = s.trim().is_empty();

    let is_too_long = s.graphemes(true).count() > 256;

    let forbidden_characters = ['/','(',')','"','<','>','\\','{','}','[',']'];
    let contains_forbidden_characters = s.chars().any(|g|forbidden_characters.contains(&g));

    ( is_empty_or_whitespace || is_too_long || contains_forbidden_characters)
}

#[tracing::instrument(
    name = "saving new subscriber details in the database",
    skip(pool,form)
)]
pub async fn insert_subscriber(pool: &PgPool, form: &FormData) -> Result<(),sqlx::Error> {
    sqlx::query!(
            r#"
                INSERT INTO subscriptions (id, email, name, subscribed_at)
                VALUES ($1,$2,$3,$4)
            "#,
            Uuid::new_v4(), form.email, form.name, Utc::now()
        )
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("failed to execute query: {:?}",e);
            e
        })?;

    Ok(())
}