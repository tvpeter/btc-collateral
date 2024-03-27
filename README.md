# Bitcoin-Collateral

BTC-COLLATERAL offers a collaborative custody service used by a lender and borrower to offer Bitcoin asset as collateral for lending. The borrower deposits their Bitcoin asset in a 2-3 multisig address, having one of the keys, the lender having one of the keys, and the service retaining one of the keys.

The lending terms between the borrower and the lender are tendered to the service. Once signed, the borrower will send assets to the 2-3 multisig address. If the borrower fulfills those terms, the lender and the borrower will sign a 2-3 multisig transaction that transfers those assets back to the borrower, otherwise, they will sign a transaction that forfeits those assets to the lender. If however, there is a dispute and any of them refuses to sign, the service will settle by signing a transaction with the party that fulfills its obligation according to the signed contract.

## Status

- PoC

## Required Dependencies

- Rust v1.74.1
- Bitcoind

## How to Run the Application

1. Clone the repository and change to the cloned directory

    ```sh
    $ git clone https://github.com/tvpeter/btc-collateral
    $ cd bitcoin-collateral
    ```
2. Rename `.env.example` to `.env`

3. Run and connect `Bitcoind`to the application
    
    After installing [Bitcoind](https://github.com/bitcoin/bitcoin/tree/master/doc), update your `.env` 
    with the nodes `RPC username, password and port`.

4. Run initial setup
    ```sh
    $ make init
    ```

5. Run unit and integration tests. Ensure all tests are passing before moving to the next step
    ```sh
    $ cargo test
    Start the server
    $ cargo run
    ```

## DB setup
- Make the `init_db.sh` file executable
```sh
$ chmod +x scripts/init_db.sh
```
- Run the `init_db.sh` file
```sh
$ ./scripts/init_db.sh
```
- Install `sqlx-cli` to manage db migrations
```sh
$ cargo install sqlx-cli --no-default-features --features postgres
```
Read more here [sqlx-cli](https://crates.io/crates/sqlx-cli)
 

## Third-party Services

1. Mempool.space API for Fees Rates

    `https://mempool.space/api/v1/fees/recommended`


## How to test

1. This tests were done in regtest

2. (a) To make it easier for you to test, we have included `test` wallet data. To make use of the data in your Bitcoin node, use the following command to load the data and wallet:
    -   `bitcoind -datadir=$(pwd)/.bitcoin` # start bitcoind 
    -   `bitcoin-cli loadwallet test` # load the wallet
    -   `bitcoin-cli getbalances`  # 6.25 btc available
    Goto step (3) to run the tests.

  (b) If you prefer to use your existing wallet, follow the below steps:
    -   Generate a new address using `bitcoin-cli getnewaddress` command.
    -   Kindly ensure that you are using unique addresses from different wallets 
    -   To switch from one wallet to another, use `bitcoin-cli loadwallet {walletname}` 
    -   To generate an address from a wallet, specify the wallet e.g `bitcoin-cli -rpcwallet={walletname} getnewaddress`
    -   To get a public key, use an already generated address and use the cli command `bitcoin-cli getadressinfo {address}` 
    -   To get unspent transaction outputs (UTXO), use the command `bitcoin-cli listunspent`
    -   Replace all addresses, public keys and txid's in the appropriate tests. Currently the following modules uses pubkeys, addresses and txids: 
        - utiils/bitcoind_rpc
        - utils/transaction_utils
        - utils/validate_address
        - utils/validate_publickeys
        - domain/funding_transaction
        - domain/generate_address
        - domain/redeeming_transaction
    -   TO get the redeemscript, run the test in the funding transaction, and it will output the redeemscript, then replace the stated redeemscript in the test_redeemscript fn to run your test.

3. Run `cargo test` to run all the tests

4. To stop the running node, use `bitcoin-cli stop`.

