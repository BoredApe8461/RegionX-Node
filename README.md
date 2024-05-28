# RegionX parachain

### Running zombienet tests

1. Install the latest zombienet release from the [Zombienet release page](https://github.com/paritytech/zombienet/releases).

2. Build the `regionx-node` by running:

    ```sh
    cargo build --release --features fast-runtime
    ```

3. Get the polkadot binary:

    ```sh
    zombienet-linux setup polkadot polkadot-parachain

    Please add the dir to your $PATH by running the command:
    export PATH=/home/<username>/RegionX-Node/:$PATH
    ```

4. Run the tests:

    - block production

        ```
        zombienet-linux -p native test ./zombienet_tests/general/block-production.zndsl
        ```

    - fee payment
        - fee payment in native tokens

            ```
            zombienet-linux -p native test ./zombienet_tests/fee-payment/native-fee-payment.zndsl
            ```

        - fee payment in custom assets

            ```
            zombienet-linux -p native test ./zombienet_tests/fee-payment/custom-fee-payment.zndsl
            ```

    - governance

        - delegated governance(relay chain token holders)

            ```
            zombienet-linux -p native test ./zombienet_tests/governance/delegated-governance.zndsl
            ```

        - native governance(RegionX token holders)

            ```
            zombienet-linux -p native test ./zombienet_tests/governance/native-governance.zndsl
            ```
    
    - cross-chain transfer

        - transfer assets
        
            ```
            zombienet-linux -p native test ./zombienet_tests/xc-transfer/asset-transfer.zndsl
            ```

        - transfer regions

            ```
            zombienet-linux -p native test ./zombienet_tests/xc-transfer/region-transfer.zndsl
            ```