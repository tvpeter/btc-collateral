use crate::startup::AppState;
use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
	username: String,
	email: String,
	phone: String,
	password: String,
}

pub async fn create_user(form: web::Form<FormData>, data: web::Data<AppState>) -> HttpResponse {
	match (sqlx::query!(
		r#"
		INSERT INTO "user"(id, username, email, phone, password_hash, created_at, updated_at)
		VALUES ($1, $2, $3, $4, $5, $6, $7);
		"#,
		Uuid::new_v4(),
		form.username,
		form.email,
		form.phone,
		form.password,
		Utc::now(),
		Utc::now()
	))
	.execute(&data.db)
	.await
	{
		Ok(_) => HttpResponse::Ok().finish(),
		Err(e) => {
			println!("Failed to execute query {} ", e);
			HttpResponse::InternalServerError().finish()
		}
	}
}
