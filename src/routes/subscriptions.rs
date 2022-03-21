use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;
use tracing::Instrument;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let id = Uuid::new_v4();

    let request_span = tracing::info_span!("Adding new subscriber",%id,%form.name,%form.email);

    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("saving new subscriber details in the database");

    match sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1,$2,$3,$4)
        "#,
        id, form.email, form.name, Utc::now()
        )
        .execute(pool.get_ref())
        .instrument(query_span)
        .await
    {
        Ok(_) => {
            HttpResponse::Ok().finish()
        },
        Err(e) => {
            tracing::error!("#{} failed to execute query. reason {:?}",id,e);
            HttpResponse::InternalServerError().finish()
        }
    }
}