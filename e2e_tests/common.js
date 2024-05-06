const RELAY_ASSET_ID = 1;

async function submitExtrinsic(signer, call, options) {
  return new Promise(async (resolve, reject) => {
    const unsub = await call.signAndSend(signer, options, (result) => {
      console.log(`Current status is ${result.status}`);
      if (result.status.isInBlock) {
        console.log(
          `Transaction included at blockHash ${result.status.asInBlock}`
        );
      } else if (result.status.isFinalized) {
        console.log(
          `Transaction finalized at blockHash ${result.status.asFinalized}`
        );
        unsub();
        return resolve();
      } else if (result.isError) {
        console.log(`Transaction error`);
        unsub();
        return reject();
      }
    });
  });
}

async function setupRelayAsset(api, signer) {
  const assetMetadata = {
    decimals: 12,
    name: "ROC",
    symbol: "ROC",
    existentialDeposit: 10n ** 3n,
    location: null,
    additional: null,
  };

  const assetSetupCalls = [
    api.tx.assetRegistry.registerAsset(assetMetadata, RELAY_ASSET_ID),
    api.tx.assetRate.create(RELAY_ASSET_ID, 1_000_000_000_000_000_000n), // 1 on 1
    api.tx.tokens.setBalance(
      signer.address,
      RELAY_ASSET_ID,
      10n ** 12n,
      0,
    ),
  ];

  const batchCall = api.tx.utility.batch(assetSetupCalls);
  const sudoCall = api.tx.sudo.sudo(batchCall);

  await submitExtrinsic(signer, sudoCall, {});
}

module.exports = { submitExtrinsic, setupRelayAsset, RELAY_ASSET_ID }
