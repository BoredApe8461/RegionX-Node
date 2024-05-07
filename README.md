# RegionX parachain

### Running zombienet tests

1. Install the latest zombienet release from the [Zombienet release page](https://github.com/paritytech/zombienet/releases).

2. Build the `regionx-node` by running:

    ```
    cargo build --release --features fast-runtime
    ```

3. Get the polkadot binary:

    ```sh
    zombienet-linux setup polkadot 

    Please add the dir to your $PATH by running the command:
    export PATH=/home/<username>/RegionX-Node/:$PATH
    ```

4. Run the tests:
 
   - block production


        ```
        zombienet-linux -p native test ./zombienet_tests/0001-block-production.zndsl
        ```

    - native fee payment

        ```
        zombienet-linux -p native test ./zombienet_tests/0002-native-fee-payment.zndsl
        ```

    - custom fee payment

        ```
        zombienet-linux -p native test ./zombienet_tests/0003-custom-fee-payment.zndsl
        ```

    - delegated governance(relay chain token holders)

        ```
        zombienet-linux -p native test ./zombienet_tests/0004-delegated-governance.zndsl
        ```

    - native governance(RegionX token holders)

        ```
        zombienet-linux -p native test ./zombienet_tests/0005-native-governance.zndsl
        ```