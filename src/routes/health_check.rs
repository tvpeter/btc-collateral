use actix_web::{HttpResponse, Responder};
use serde::Serialize;

#[derive(Serialize)]
struct HealthCheckResponse {
	message: String,
}

pub async fn health_check() -> impl Responder {
	let health_check_response = HealthCheckResponse {
		message: String::from("OK"),
	};

	HttpResponse::Ok().json(health_check_response)
}
