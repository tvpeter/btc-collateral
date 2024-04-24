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
		let sighash = sighash_cache.p2wsh_signature_hash(
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

#[cfg(test)]
mod tests {
	use crate::
		domain::MultisigAddress
	;
	use bitcoin::{
		bip32::Xpriv, secp256k1, AddressType, Network::Regtest, PrivateKey,
		PublicKey,
	};


	fn get_xprivs() -> (Xpriv, Xpriv, Xpriv) {
		(
		 Xpriv::new_master(Regtest, Default::default()).unwrap(),
		 Xpriv::new_master(Regtest, Default::default()).unwrap(),
		 Xpriv::new_master(Regtest, Default::default()).unwrap(),
		)
	}

	fn get_privkeys()-> (PrivateKey, PrivateKey, PrivateKey)  {
		let (x_priv_a, x_priv_b, x_priv_c) = get_xprivs();

		(
			x_priv_a.to_priv(),
			x_priv_b.to_priv(),
			x_priv_c.to_priv(),
		)
	}

	fn derive_address() -> MultisigAddress {
		let (privkey_a, privkey_b, privkey_c) = get_privkeys();

		let secp_a = secp256k1::Secp256k1::new();
		let pubkey_a = PublicKey::from_private_key(&secp_a, &privkey_a);

		let secp_b = secp256k1::Secp256k1::new();
		let pubkey_b = PublicKey::from_private_key(&secp_b, &privkey_b);

		let secp_c = secp256k1::Secp256k1::new();
		let pubkey_c = PublicKey::from_private_key(&secp_c, &privkey_c);

		MultisigAddress {
			lender_pubkey: pubkey_a,
			borrower_pubkey: pubkey_b,
			service_pubkey: pubkey_c,
		}
	}

	#[test]
	fn generate_address() {
		let parties_keys = derive_address();
		let address = parties_keys.create_p2wsh_address();

		assert_eq!(address.address_type(), Some(AddressType::P2wsh));
		assert_eq!(address.network(), &Regtest);
	}	
}
