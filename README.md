# Bitcoin-Collateral

BTC-COLLATERAL offers a collaborative custody service that both the lender and the borrower can use to offer as collateral for lending. The borrower will deposit their asset in a 2-3 multisig address. The borrower will have one of the keys, the lender will have one of the keys, and the service will retain one of the keys.

The lending terms between the borrower and the lender are tendered to the service. Once signed, the borrower will send assets to the 2-3 multisig address. If the borrower fulfills those terms, the lender and the borrower will sign a 2-3 multisig transaction that transfers those assets back to the borrower, otherwise, they will sign a transaction that forfeits those assets to the lender. If however, there is a dispute and any of them refuses to sign, the service will settle by signing a transaction with the party that fulfills its obligation according to the signed contract.

## MVP Status
- Work in progess. 

## Required Dependencies
- Rust v1.74.1 

## Run
1. Clone the repository and change to the cloned directory
```sh
$ git clone https://github.com/tvpeter/btc-collateral
$ cd bitcoin-collateral
```
2. Run unit and integration tests. Ensure all tests are passing before moving to the next step
```sh
$ cargo test
```
4. Start the server
```sh
$ cargo run
```
