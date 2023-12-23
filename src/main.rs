mod domain;

use std::str::FromStr;

use bitcoin::PublicKey;
use domain::generate_address::PartiesPublicKeys;

fn main() {
    let pubkey_string = "0347ff3dacd07a1f43805ec6808e801505a6e18245178609972a68afbc2777ff2b";
    let borrower_pubkey = PublicKey::from_str(pubkey_string).expect("pubkey");

    let lender_pubkey = PublicKey::from_str(
        "02ba604e6ad9d3864eda8dc41c62668514ef7d5417d3b6db46e45cc4533bff001c",
    )
    .expect("pubkey");

    let service_pubkey = PublicKey::from_str("03df154ebfcf29d29cc10d5c2565018bce2d9edbab267c31d2caf44a63056cf99f").expect("pubkey");

    let combined_keys = PartiesPublicKeys::new(borrower_pubkey, lender_pubkey, service_pubkey);

    // let redeem_script_hex = combined_keys.redeem_script_hex();

    // println!("The redeem script hex: {:?}", redeem_script_hex);

    let address = combined_keys.create_p2sh_address();

    println!("p2sh address: {:?}", address);

}


