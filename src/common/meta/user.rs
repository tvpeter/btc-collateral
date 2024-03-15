use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

static USERNAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[0-9A-Za-z_]+$").unwrap());

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct CreateUserData {
	#[validate(
		regex(
			code = "username",
			path = "USERNAME_REGEX",
			message = "Invalid username"
		),
		length(
			min = 3,
			max = 16,
			message = "Username must be between 3 and 16 characters"
		)
	)]
	pub username: String,
	#[validate(email(code = "email", message = "Email is invalid"))]
	pub email: String,
	pub phone: String,
	#[validate(length(min = 8, max = 32))]
	pub password: String,
}

#[derive(Debug, Clone, Serialize, Default, Deserialize, FromRow)]
pub struct User {
	pub id: Uuid,
	pub username: String,
	pub email: String,
	pub phone: Option<String>,
	pub created_at: DateTime<Utc>,
}

#[derive(Debug, Validate, Deserialize, Serialize)]
pub struct UpdatePasswordRequest {
	pub id: Uuid,
	pub current_password: String,
	#[validate(length(min = 8, max = 32,))]
	pub new_password: String,
	#[validate(length(min = 8, max = 32))]
	#[validate(must_match(
		code = "confirm_password",
		message = "Passwords do not match",
		other = "new_password"
	))]
	pub confirm_password: String,
}
