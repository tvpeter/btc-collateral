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

## Third-party Services

1. Mempool.space API for Fees Rates

    `https://mempool.space/api/v1/fees/recommended`
