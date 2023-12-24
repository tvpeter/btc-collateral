use std::fmt::format;

use hex;
use bitcoin::{Address, Network, Script, PublicKey};
use bitcoin::address::Error;

use crate::utils::validate_publickeys::{is_valid_pubkey, is_hex};


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

    //OP_2 [pubkey1] [pubkey2] [pubkey3] OP_3 OP_CHECKMULTISIG 
    pub fn redeem_script_hex(&self) -> String {
        self.validate_publickeys();

        let borrower_pubkey_len = format!("{:x}", &self.borrower_pubkey.to_bytes().len());
        let borrower_pubkey_hex = hex::encode(self.borrower_pubkey.to_string());

        let lender_pubkey_len = format!("{:x}", &self.lender_pubkey.to_bytes().len());
        let lender_pubkey_hex = hex::encode(self.lender_pubkey.to_string());
        
        let service_pubkey_len = format!("{:x}", &self.service_pubkey.to_bytes().len());
        let service_pubkey_hex = hex::encode(self.service_pubkey.to_string());
        "52".to_string() + &borrower_pubkey_len + &borrower_pubkey_hex + &lender_pubkey_len + &lender_pubkey_hex + &service_pubkey_len + &service_pubkey_hex + "53ae"
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
    let pubkey_string = "0347ff3dacd07a1f43805ec6808e801505a6e18245178609972a68afbc2777ff2b";
    let borrower_pubkey = PublicKey::from_str(pubkey_string).expect("pubkey");

    let lender_pubkey = PublicKey::from_str(
        "02ba604e6ad9d3864eda8dc41c62668514ef7d5417d3b6db46e45cc4533bff001c",
    )
    .expect("pubkey");

    let service_pubkey = PublicKey::from_str("03df154ebfcf29d29cc10d5c2565018bce2d9edbab267c31d2caf44a63056cf99f").expect("pubkey");

    let combined_keys = PartiesPublicKeys::new(borrower_pubkey, lender_pubkey, service_pubkey);

    let redeem_script_hex = combined_keys.redeem_script_hex();

    println!("The redeem script hex: {:?}", redeem_script_hex);

   }

   
}

