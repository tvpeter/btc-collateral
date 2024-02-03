use anyhow::Result;
use bdk::blockchain::ElectrumBlockchain;
use bdk::database::SqliteDatabase;
use bdk::electrum_client::Client;
use bdk::keys::{
	bip39::{Language, Mnemonic, WordCount},
	DerivableKey, ExtendedKey, GeneratableKey, GeneratedKey,
};
use bdk::template::Bip84;
use bdk::{miniscript, KeychainKind};
use bdk::{SyncOptions, Wallet};
use dotenv::dotenv;
use std::env;
use std::path::Path;

/// Generates a 12 word mnemonic
pub fn derive_mnemonic() -> Result<String> {
	let mnemonic: GeneratedKey<_, miniscript::Segwitv0> =
		Mnemonic::generate((WordCount::Words12, Language::English)).unwrap();
	let mnemonic_words = mnemonic.to_string();
	Ok(mnemonic_words)
}

pub fn setup_wallet(mnemonic: Option<String>) -> Result<Wallet<SqliteDatabase>, anyhow::Error> {
	dotenv().ok();
	let network = env::var("NETWORK").unwrap();
	let mnemonic_words = match mnemonic {
		Some(mnemonic) => mnemonic,
		None => derive_mnemonic()?,
	};
	let mnemonic = Mnemonic::parse(mnemonic_words).unwrap();
	// Generate the extended key
	let xkey: ExtendedKey = mnemonic.into_extended_key().unwrap();
	// Get xprv from the extended key
	let xprv = xkey.into_xprv(network.parse()?).unwrap();
	let db_path: &Path = Path::new("wallet.db");

	let wallet = match Wallet::new(
		Bip84(xprv, KeychainKind::External),
		Some(Bip84(xprv, KeychainKind::Internal)),
		network.parse()?,
		SqliteDatabase::new(db_path),
	) {
		Ok(wallet) => wallet,
		Err(e) => {
			println!("Failed to set up wallet: {}", e);
			println!("Error: {:?}", e);
			return Err(anyhow::Error::msg("Failed to set up wallet"));
		}
	};
	Ok(wallet)
}

pub fn sync_wallet(wallet: &Wallet<SqliteDatabase>) -> Result<(), anyhow::Error> {
	let blockchain = ElectrumBlockchain::from(Client::new("127.0.0.1:60401")?);
	wallet.sync(&blockchain, SyncOptions::default())?;
	Ok(())
}
