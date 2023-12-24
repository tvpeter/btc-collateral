mod domain;
mod utils;

use std::str::FromStr;

use bitcoin::PublicKey;
use domain::generate_address::PartiesPublicKeys;

fn main() {
    let pubkey_string = "02f0eaa04e609b0044ef1fe09a350dc4b744a5a8604a6fa77bc9bf6443ea50739f";
    let borrower_pubkey = PublicKey::from_str(pubkey_string).expect("invalid borrower pubkey");

    let lender_pubkey = PublicKey::from_str(
        "037c60db011a840523f216e7198054ef071c5acd3d4b466cf2658b7faf30c11e33",
    )
    .expect("invalid lender pubkey");


    let service_pubkey = PublicKey::from_str("02ca49f36d3de1e135e033052611dd0873af55b57f07d5d0d1090ceb267ac34e6b").expect("invalid service pubkey");

    let combined_keys = PartiesPublicKeys::new(borrower_pubkey, lender_pubkey, service_pubkey);

    combined_keys.print_addresses();

}



