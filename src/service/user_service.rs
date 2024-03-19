use crate::{
	common::meta::user::UpdatePasswordRequest, startup::AppState, utils::api_result::ApiResult,
};
use actix_web::{web, HttpResponse};
use reqwest::StatusCode;
use crate::common::meta::user::{CreateUserData, User};
use chrono::Utc;
use secrecy::ExposeSecret;
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
	let password_hash = crate::utils::password::hash_password(form.password.clone()).unwrap();

	match (sqlx::query!(
		r#"
		INSERT INTO "user"(id, username, email, phone, password_hash, created_at, updated_at)
		VALUES ($1, $2, $3, $4, $5, $6, $7);
		"#,
		Uuid::new_v4(),
		form.username,
		form.email,
		form.phone,
		password_hash.expose_secret(),
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

pub async fn get_user_by_id(id: web::Path<Uuid>, data: web::Data<AppState>) -> HttpResponse {
	let user_id = fetch_user_id(id.into_inner(), data);

	match user_id.await {
		Ok(user_id) => HttpResponse::Ok().json(ApiResult::success(Some(user_id))),
		Err(e) => {
			println!("Failed to fetch user {}", e);
			HttpResponse::InternalServerError().finish()
		}
	}
}
/*
pub async fn update_user(id: web::Path<Uuid>, data: web::Data<AppState>) -> HttpResponse {
	match sqlx::query!(
		r#"
		UPDATE "user"
		SET updated_at = $1
		WHERE id = $2;
		"#,
		Utc::now(),
		id.into_inner()
	)
	.execute(&data.db)
	.await
	{
		Ok(_) => HttpResponse::Ok().finish(),
		Err(e) => {
			println!("Failed to update user {}", e);
			HttpResponse::InternalServerError().finish()
		}
	}
}
*/
pub async fn update_password(
	form: web::Form<UpdatePasswordRequest>,
	data: web::Data<AppState>,
) -> HttpResponse {
	match fetch_user_id(form.id, data.clone()).await {
		Ok(user) => {
			if user.id != form.id {
				return HttpResponse::BadRequest().json(ApiResult::<()>::error(
					StatusCode::BAD_REQUEST.as_u16().to_string(),
					Some("Invalid user".to_string()),
				));
			}
		}
		Err(e) => {
			println!("Failed to fetch user {}", e);
			return HttpResponse::BadRequest().json(ApiResult::<()>::error(
				StatusCode::BAD_REQUEST.as_u16().to_string(),
				Some("Invalid user".to_string()),
			));
		}
	}

	match sqlx::query!(
		r#"
		UPDATE "user"
		SET password_hash = $1, updated_at = $2
		WHERE id = $3;
		"#,
		form.new_password.expose_secret(),
		Utc::now(),
		form.id
	)
	.execute(&data.db)
	.await
	{
		Ok(_) => HttpResponse::Ok().finish(),
		Err(e) => {
			println!("Failed to update user {}", e);
			HttpResponse::InternalServerError().finish()
		}
	}
}

pub async fn delete_user(id: web::Path<Uuid>, data: web::Data<AppState>) -> HttpResponse {
	match sqlx::query!(
		r#"
		DELETE FROM "user"
		WHERE id = $1;
		"#,
		id.into_inner()
	)
	.execute(&data.db)
	.await
	{
		Ok(_) => HttpResponse::Ok().finish(),
		Err(e) => {
			println!("Failed to delete user {}", e);
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

pub async fn fetch_user_id(id: Uuid, data: web::Data<AppState>) -> Result<User, anyhow::Error> {
	let row = sqlx::query_as!(
		User,
		r#"
        SELECT id, username, email, phone, created_at
        FROM "user"
        WHERE id = $1
        "#,
		id,
	)
	.fetch_one(&data.db)
	.await?;
	Ok(row)
}
