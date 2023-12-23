use bitcoin::{Address, Network, Script, PublicKey};
use bitcoin::address::Error;

use crate::utils::validate_publickeys::is_valid_pubkey;


#[derive(Debug, Clone)]
pub struct PartiesPublicKeys {
    pub borrower_pubkey: PublicKey,
    pub lender_pubkey: PublicKey,
    pub service_pubkey: PublicKey,
}

impl PartiesPublicKeys {
    pub fn new(borrower_pubkey: PublicKey, lender_pubkey: PublicKey, service_pubkey: PublicKey) -> Self {
        Self {
            borrower_pubkey,
            lender_pubkey,
            service_pubkey,
        }
    }

    fn validate_publickeys(&self) {
        if !is_valid_pubkey( &self.borrower_pubkey.to_bytes()) {
            panic!("Invalid borrower public key");
        }

        if !is_valid_pubkey( &self.lender_pubkey.to_bytes()) {
            panic!("Invalid lender public key");
        }

        if !is_valid_pubkey( &self.service_pubkey.to_bytes()) {
            panic!("Invalid service public key");
        }
    }


    //OP_2  [pubkey1] [pubkey2] [pubkey3] OP_3 OP_CHECKMULTISIG 
    pub fn redeem_script_hex(&self) -> String {
        self.validate_publickeys();

        let borrower_pubkey_len = format!("{:x}", &self.borrower_pubkey.to_bytes().len());
        let lender_pubkey_len = format!("{:x}", &self.lender_pubkey.to_bytes().len());
        let service_pubkey_len = format!("{:x}", &self.service_pubkey.to_bytes().len());

        "52".to_string() + &borrower_pubkey_len + &self.borrower_pubkey.to_string() + &lender_pubkey_len + &self.lender_pubkey.to_string() + &service_pubkey_len + &self.service_pubkey.to_string() + "53ae"
    }

    pub fn create_p2sh_address(&self) -> Result<Address, String> {
        let binding = self.redeem_script_hex();
        let redeemscript_bytes = binding.as_bytes();
        let derived_script = Script::from_bytes(redeemscript_bytes);
        let generated_address = Address::p2sh(derived_script, Network::Regtest);
        generated_address.map_err(|err| format!("Error creating p2sh address: {:?}", err))
    }
    
    pub fn create_p2wsh_address(&self) -> Address {
        let binding = self.redeem_script_hex();
        let redeemscript_bytes = binding.as_bytes();
        let redeem_script = Script::from_bytes(redeemscript_bytes);
        Address::p2wsh(redeem_script, Network::Regtest)
    }

    pub fn print_addresses(&self) {
        let p2sh_address = self.create_p2sh_address();
        let _derived_address = match p2sh_address {
            Ok(generated_address) => {
                if generated_address.is_spend_standard() {
                    println!("P2SH address: {}", generated_address);
                } else {
                    println!("{} is a non-standard address", generated_address);
                }
                Ok(())  // Returning Ok(()) to match the Result type
            }
            Err(_) => Err(Error::UnrecognizedScript),
        };

        let p2wsh_address = self.create_p2wsh_address();
        println!("P2WSH address: {:?}", p2wsh_address);

    }

}


#[cfg(test)]
mod tests {

use std::str::FromStr;

use super::*;

   #[test]
   fn test_redeem_script_hex(){
    let pubkey_string = "02f0eaa04e609b0044ef1fe09a350dc4b744a5a8604a6fa77bc9bf6443ea50739f";
    let borrower_pubkey = PublicKey::from_str(pubkey_string).expect("Invalid borrower pubkey");

    let lender_pubkey = PublicKey::from_str(
        "037c60db011a840523f216e7198054ef071c5acd3d4b466cf2658b7faf30c11e33",
    )
    .expect("Invalid lender pubkey");

    let service_pubkey = PublicKey::from_str("02ca49f36d3de1e135e033052611dd0873af55b57f07d5d0d1090ceb267ac34e6b").expect("Invalid service pubkey");

    let combined_keys = PartiesPublicKeys::new(borrower_pubkey, lender_pubkey, service_pubkey);

    let redeem_script_hex = combined_keys.redeem_script_hex();

    println!("The redeem script hex: {:?}", redeem_script_hex);

   }

   
}


