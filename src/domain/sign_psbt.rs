use anyhow::{anyhow, Context, Result};
use bitcoin::bip32::{DerivationPath, Xpriv};
use bitcoin::ecdsa::Signature;
use bitcoin::psbt::{Input, Psbt};
use bitcoin::secp256k1::{self, All, Message, Secp256k1};
use bitcoin::sighash::SighashCache;
use bitcoin::{EcdsaSighashType, PublicKey};

pub fn set_sighash_type(signature: secp256k1::ecdsa::Signature, input: &Input) -> Signature {
	let sighash_type = get_sighash_type(input);
	Signature {
		sig: signature,
		hash_ty: sighash_type,
	}
}
pub fn get_sighash_type(input: &Input) -> EcdsaSighashType {
	input
		.sighash_type
		.and_then(|t| t.ecdsa_hash_ty().ok())
		.unwrap_or(EcdsaSighashType::All)
}
pub fn get_partial_derivation(
	derivation: &DerivationPath,
	sub_derivation: &DerivationPath,
) -> Result<DerivationPath> {
	if derivation.len() > sub_derivation.len() {
		return Err(anyhow!(
			"Can't get a partial derivation from a derivation greater than the sub derivation"
		));
	}
	let partial = &sub_derivation[derivation.len()..];
	dbg!(&partial);
    Ok(DerivationPath::from(partial))
}
pub fn derive_relative_xpriv(
	xprv: &Xpriv,
	secp: &Secp256k1<All>,
	derivation: &DerivationPath,
	sub_derivation: &DerivationPath,
) -> Result<Xpriv> {
	xprv.derive_priv(secp, &get_partial_derivation(derivation, sub_derivation)?)
		.map_err(|e| anyhow!("{e}"))
}
pub fn sign_psbt(mut psbt: Psbt, xprv: Xpriv, derivation: &DerivationPath) -> Result<Psbt> {
	let secp = Secp256k1::new();
	for (index, input) in psbt.inputs.iter_mut().enumerate() {
		let witness_script = input
			.witness_script
			.as_ref()
			.context("Missing witness script")?;
		let amount = input
			.witness_utxo
			.as_ref()
			.context("Witness utxo not found")?
			.value;
		let mut sighash_cache = SighashCache::new(&psbt.unsigned_tx);
		let sighash = sighash_cache.p2wpkh_signature_hash(
			index,
			witness_script,
			amount,
			get_sighash_type(input),
		)?;
		let mut input_keypairs = Vec::new();
		for (_, (fingerprint, sub_derivation)) in input.bip32_derivation.iter() {
			if fingerprint != &xprv.fingerprint(&secp) {
				continue;
			}
			let parent_xprv = derive_relative_xpriv(&xprv, &secp, derivation, sub_derivation)?;
			input_keypairs.push(parent_xprv.to_keypair(&secp));
		}
		if input_keypairs.is_empty() {
			return Err(anyhow!("No private keys to sign this psbt"));
		}
		for keypair in input_keypairs {
			let message = &Message::from_digest_slice(sighash.to_string().as_bytes())?;
			let signature = secp.sign_ecdsa(message, &keypair.secret_key());
			input.partial_sigs.insert(
				PublicKey::new(keypair.public_key()),
				set_sighash_type(signature, input),
			);
			secp.verify_ecdsa(message, &signature, &keypair.public_key())?;
		}
	}
	Ok(psbt)
}
