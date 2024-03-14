use crate::{startup::AppState, utils::api_result::ApiResult};
use actix_web::{web, HttpResponse};
use reqwest::StatusCode;
// use sql_query_builder as sql;
use crate::common::meta::user::{CreateUserData, User};
use chrono::Utc;
use std::string::String;
use uuid::Uuid;
use validator::Validate;

// https://www.ibm.com/docs/en/db2-for-zos/11?topic=codes-sqlstate-values-common-error
pub const PG_UNIQUE_VIOLATION_ERROR: &str = "23505";

pub async fn create_user(
	form: web::Form<CreateUserData>,
	data: web::Data<AppState>,
) -> HttpResponse {
	match form.validate() {
		Ok(_) => (),
		Err(e) => {
			println!("Failed to validate form {}", e);
			return HttpResponse::BadRequest().json(ApiResult::<()>::error(
				StatusCode::BAD_REQUEST.as_u16().to_string(),
				Some(e.to_string()),
			));
		}
	}
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
		Ok(_) => HttpResponse::Ok().json(ApiResult::success(None::<()>)),

		Err(sqlx::Error::Database(dberr)) => {
			if let Some(code) = dberr.code() {
				print!("Error code {:?}", code);
				if code == PG_UNIQUE_VIOLATION_ERROR {
					let msg = dberr.message();
					if msg.contains("username_key") {
						return HttpResponse::BadRequest().json(ApiResult::<()>::error(
							StatusCode::BAD_REQUEST.as_u16().to_string(),
							Some("Username already exists".to_string()),
						));
					} else if msg.contains("email_key") {
						return HttpResponse::BadRequest().json(ApiResult::<()>::error(
							StatusCode::BAD_REQUEST.as_u16().to_string(),
							Some("Email already exists".to_string()),
						));
					} else if msg.contains("phone_key") {
						return HttpResponse::BadRequest().json(ApiResult::<()>::error(
							StatusCode::BAD_REQUEST.as_u16().to_string(),
							Some("Phone already exists".to_string()),
						));
					} else {
						println!("Failed to execute query {:?} ", dberr);
						return HttpResponse::InternalServerError().finish();
					}
				}
				return HttpResponse::BadRequest().json(ApiResult::<()>::error(
					StatusCode::BAD_REQUEST.as_u16().to_string(),
					Some(code.to_string()),
				));
			}
			// Err::<(), _>(sqlx::Error::Database(dberr));
			HttpResponse::InternalServerError().finish()
		}
		Err(e) => {
			println!("Failed to execute query {} ", e);
			HttpResponse::InternalServerError().finish()
		}
	}
}

pub async fn list_users(data: web::Data<AppState>) -> HttpResponse {
	match sqlx::query_as!(
		User,
		r#"
		SELECT id, username, email, phone, created_at
		FROM "user";
		"#
	)
	.fetch_all(&data.db)
	.await
	{
		Ok(users) => {
			print!("Fetched users {:?}", users);
			HttpResponse::Ok().json(ApiResult::success(Some(users)))
		}
		Err(e) => {
			println!("Failed to fetch users {}", e);
			HttpResponse::InternalServerError().finish()
		}
	}
}

pub async fn fetch_users(data: web::Data<AppState>) -> Result<Vec<User>, anyhow::Error> {
	let users = sqlx::query_as::<_, User>(
		r#"
		SELECT id, username, email, phone, created_at
		FROM "user";
		"#,
	)
	.fetch_all(&data.db)
	.await?;
	Ok(users)
}
