use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version};
use bitcoin::key::rand;
use secrecy::{ExposeSecret, Secret};
pub fn hash_password(password: Secret<String>) -> Result<Secret<String>, anyhow::Error> {
	let salt = SaltString::generate(&mut rand::thread_rng());
	let password_hash = Argon2::new(
		Algorithm::Argon2id,
		Version::V0x13,
		Params::new(15000, 2, 1, None).unwrap(),
	)
	.hash_password(password.expose_secret().as_bytes(), &salt)?
	.to_string();
	Ok(Secret::new(password_hash))
}
