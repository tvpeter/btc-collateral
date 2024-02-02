use crate::startup::AppState;
use actix_web::error::InternalError;
use actix_web::{web, HttpResponse, Responder};
use bdk::wallet::AddressIndex;
use serde::{Deserialize, Serialize};
use wallet::{derive_mnemonic, setup_wallet, sync_wallet};

#[derive(Debug, Serialize)]
struct GenerateMnemonicResponse {
	mnemonic: String,
}

pub async fn generate_mnemonic(data: web::Data<AppState>) -> impl Responder {
	let mnemonic = derive_mnemonic().unwrap_or_else(|e| {
		{ InternalError::from_response(e, HttpResponse::InternalServerError().finish()) }
			.to_string()
	});
	let mut data = data.passkey.lock().unwrap();
	*data = mnemonic.clone();
	let mnemonic = GenerateMnemonicResponse { mnemonic };
	HttpResponse::Ok().json(mnemonic)
}

#[derive(Deserialize)]
pub struct FormData {
	mnemonic: Option<String>,
}

pub async fn create_or_recover_wallet(
	_form: web::Form<FormData>,
	data: web::Data<AppState>,
) -> HttpResponse {
	let secret_key = match Some(_form.mnemonic.clone()) {
		Some(secret_key) => secret_key,
		None => {
			let secret_mutex = &data.passkey.lock().unwrap();
			Some(secret_mutex.to_string())
		}
	};
	let wallet = match setup_wallet(secret_key.clone()) {
		Ok(wallet) => wallet,
		Err(e) => {
			panic!("Failed to set up wallet: {}", e)
		}
	};
	let mut app_wallet = data.wallet.lock().unwrap();
	*app_wallet = wallet;
	HttpResponse::Ok().json(serde_json::json!({"name": secret_key }))
}

pub async fn get_address(data: web::Data<AppState>) -> impl Responder {
	let address = data.wallet.lock().unwrap().get_address(AddressIndex::New);
	let address = format!("{}", address.unwrap());
	HttpResponse::Ok().json(serde_json::json!({ "address": address }))
}

pub async fn get_balance(data: web::Data<AppState>) -> impl Responder {
	let wallet = data.wallet.lock().unwrap();
	sync_wallet(&wallet).unwrap();
	let balance = wallet.get_balance().unwrap();
	HttpResponse::Ok().json(serde_json::json!({ "balance": balance }))
}
