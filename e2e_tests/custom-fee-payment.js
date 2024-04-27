const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const { submitExtrinsic } = require("./common");

const RELAY_ASSET_ID = 1;

async function run(nodeName, networkInfo, _jsArgs) {
  const { wsUri: regionXUri } = networkInfo.nodesByName[nodeName];
  const { wsUri: rococoUri } = networkInfo.nodesByName["rococo-validator01"];

  const rococoApi = await ApiPromise.create({
    provider: new WsProvider(rococoUri),
  });
  const regionXApi = await ApiPromise.create({
    provider: new WsProvider(regionXUri),
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

  const setXcmVersion = rococoApi.tx.xcmPallet.forceDefaultXcmVersion([3]);
  await submitExtrinsic(alice, rococoApi.tx.sudo.sudo(setXcmVersion), {});

  const assetMetadata = {
    decimals: 12,
    name: "ROC",
    symbol: "ROC",
    existentialDeposit: 10n ** 3n,
    location: null,
    additional: null,
  };

  const assetSetupCalls = [
    regionXApi.tx.assetRegistry.registerAsset(assetMetadata, RELAY_ASSET_ID),
    regionXApi.tx.assetRate.create(RELAY_ASSET_ID, 1_000_000_000_000_000_000n), // 1 on 1
    regionXApi.tx.tokens.setBalance(
      alice.address,
      RELAY_ASSET_ID,
      10n ** 12n,
      0,
    ),
  ];
  const batchCall = regionXApi.tx.utility.batch(assetSetupCalls);
  const sudoCall = regionXApi.tx.sudo.sudo(batchCall);

  await submitExtrinsic(alice, sudoCall, {});

  const receiverKeypair = new Keyring();
  receiverKeypair.addFromAddress(alice.address);

  const feeAssetItem = 0;
  const weightLimit = "Unlimited";
  const reserveTransfer = rococoApi.tx.xcmPallet.limitedReserveTransferAssets(
    { V3: { parents: 0, interior: { X1: { Parachain: 2000 } } } }, //dest
    {
      V3: {
        parents: 0,
        interior: {
          X1: {
            AccountId32: {
              chain: "Any",
              id: receiverKeypair.pairs[0].publicKey,
            },
          },
        },
      },
    }, //beneficiary
    {
      V3: [
        {
          id: {
            Concrete: { parents: 0, interior: "Here" },
          },
          fun: {
            Fungible: 10n ** 9n,
          },
        },
      ],
    }, //asset
    feeAssetItem,
    weightLimit,
  );
  await submitExtrinsic(alice, reserveTransfer, {});

  // Try to pay for fees with relay chain asset.
  const remarkCall = regionXApi.tx.system.remark("0x44");
  await submitExtrinsic(alice, remarkCall, { assetId: RELAY_ASSET_ID });
}

module.exports = { run };
