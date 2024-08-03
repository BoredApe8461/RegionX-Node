# RegionX parachain

### Running zombienet tests

1. Install the latest zombienet release from the [Zombienet release page](https://github.com/paritytech/zombienet/releases).

2. Build the `regionx-node` by running:

    ```sh
    cargo build --release --features fast-runtime
    ```

3. Install dependencies:  
    ```
    npm i
    ```

4. Run the tests:

    - block production

        ```
        npm run test -- ./zombienet_tests/general/block-production.zndsl
        ```

    - fee payment
        - fee payment in native tokens

            ```
            npm run test -- ./zombienet_tests/fee-payment/native-fee-payment.zndsl
            ```

        - fee payment in custom assets

            ```
            npm run test -- ./zombienet_tests/fee-payment/custom-fee-payment.zndsl
            ```

    - governance

        - delegated governance(relay chain token holders)

            ```
            npm run test -- ./zombienet_tests/governance/delegated-governance.zndsl
            ```

        - native governance(RegionX token holders)

            ```
            npm run test -- ./zombienet_tests/governance/native-governance.zndsl
            ```
    
    - cross-chain transfer

        - transfer assets
        
            ```
            npm run test -- ./zombienet_tests/xc-transfer/asset-transfer.zndsl
            ```

        - transfer regions

            ```
            npm run test -- ./zombienet_tests/xc-transfer/region-transfer.zndsl
            ```

    - order tests

        - processing
        
            ```
            npm run test -- ./zombienet_tests/order/processing.zndsl
            ```
