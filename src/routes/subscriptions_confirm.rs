use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(
    name = "confirm a pending subscriber",
    skip(_parameters,pool)
)]
pub async fn confirm(_parameters: web::Query<Parameters>, pool : web::Data<PgPool>) -> HttpResponse {
    // let id = match get_subscriber_id_from_token(&po)

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "get subscriber token from id",
    skip(subscription_token, pool)
)]
pub async fn get_subscriber_id_from_token(pool: &PgPool, subscription_token: &str) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
            r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
            subscription_token
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("cound not execute query. {:?}",e);

            e
        })?;

    Ok(result.map(|r| r.subscriber_id))
}