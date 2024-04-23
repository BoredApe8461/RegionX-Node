# RegionX parachain

### Running zombienet tests

1. Install the latest zombienet release from the [Zombienet release page](https://github.com/paritytech/zombienet/releases).

2. Build the `regionx-node` by running:

    ```
    cargo build --release
    ```

3. Get the polkadot binary:

    ```sh
    zombienet-linux setup polkadot 

    Please add the dir to your $PATH by running the command:
    export PATH=/home/<username>/zombienet/dist:$PATH
    ```

4. Run the test:

    ```
    zombienet-linux -p native test ./zombienet_tests/0001-smoke-test.zndsl
    ```
