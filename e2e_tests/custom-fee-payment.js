const { ApiPromise, WsProvider } = require("@polkadot/api");
const { submitExtrinsic } = require("./common");

const ASSET_ID = 42;

async function run(nodeName, networkInfo, _jsArgs) {
  const { wsUri } = networkInfo.nodesByName[nodeName];
  const api = await ApiPromise.create({
    provider: new WsProvider(wsUri),
    signedExtensions: {
      ChargeAssetTxPayment: {
        extrinsic: {
          tip: "Compact<Balance>",
          assetId: "Option<AssetId>",
        },
        payload: {},
      },
    },
  });

  // account to submit tx
  const keyring = new zombie.Keyring({ type: "sr25519" });
  const alice = keyring.addFromUri("//Alice");

  const assetMetadata = {
    decimals: 10,
    name: "DOT",
    symbol: "DOT",
    existentialDeposit: 10n**3n,
    location: null,
    additional: null
  };

  const assetSetupCalls = [
    api.tx.assetRegistry.registerAsset(assetMetadata, ASSET_ID),
    api.tx.assetRate.create(ASSET_ID, 1000000000000000000n), // 1 on 1
    api.tx.tokens.setBalance(alice.address, ASSET_ID, 10n**12n, 0),
  ];
  const batchCall = api.tx.utility.batch(assetSetupCalls);
  const sudo = api.tx.sudo.sudo(batchCall);

  await submitExtrinsic(alice, sudo, {});

  const remarkCall = api.tx.system.remark("0x44");
  await submitExtrinsic(alice, remarkCall, {assetId: ASSET_ID});
}

module.exports = { run };
