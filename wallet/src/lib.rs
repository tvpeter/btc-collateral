use anyhow::Result;
use bdk::database::SqliteDatabase;
use bdk::keys::{
	bip39::{Language, Mnemonic, WordCount},
	DerivableKey, ExtendedKey, GeneratableKey, GeneratedKey,
};
use bdk::template::Bip84;
use bdk::wallet::AddressIndex;
use bdk::{bitcoin::Network, Wallet};
use bdk::{miniscript, KeychainKind};
use std::path::Path;

pub fn create_wallet() -> Result<(), anyhow::Error> {
	let network = Network::Regtest; // Or this can be Network::Bitcoin, Network::Signet or Network::Regtest

	// Generate fresh mnemonic
	let mnemonic: GeneratedKey<_, miniscript::Segwitv0> =
		Mnemonic::generate((WordCount::Words12, Language::English)).unwrap();
	// Convert mnemonic to string
	let mnemonic_words = mnemonic.to_string();
	// Parse a mnemonic
	let mnemonic = Mnemonic::parse(mnemonic_words).unwrap();
	// Generate the extended key
	let xkey: ExtendedKey = mnemonic.into_extended_key().unwrap();
	// Get xprv from the extended key
	let xprv = xkey.into_xprv(network).unwrap();

	let db_path: &Path = Path::new("wallet.db");

	let wallet = match Wallet::new(
		Bip84(xprv, KeychainKind::External),
		Some(Bip84(xprv, KeychainKind::Internal)),
		network,
		SqliteDatabase::new(db_path),
	) {
		Ok(wallet) => wallet,
		Err(e) => {
			println!("Failed to set up wallet: {}", e);
			println!("Error: {:?}", e);
			return Err(anyhow::Error::msg("Failed to set up wallet"));
		}
	};

	let balance = wallet.get_balance()?;
	dbg!(balance);
	let address = wallet.get_address(AddressIndex::New)?;
	dbg!(address);

	Ok(())
}
