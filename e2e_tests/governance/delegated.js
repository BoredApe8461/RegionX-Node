const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const { submitExtrinsic, setupRelayAsset, RELAY_ASSET_ID } = require("../common");

const PREIMAGE_HASH = "0xb8375f7ca0c64a384f2dd643a0d520977f3aae06e64afb8c960891eee5147bd1";

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
  const keyring = new Keyring({ type: "sr25519" });
  const alice = keyring.addFromUri("//Alice");
  const anna = keyring.addFromUri("//Anna");

  await setupRelayAsset(api, alice);

  const giveBalanceCall = api.tx.tokens.setBalance(anna.address, RELAY_ASSET_ID, 10n ** 18n, 0);
  await submitExtrinsic(alice, api.tx.sudo.sudo(giveBalanceCall), {});

  const remarkCallBytes = api.tx.system.remark("hey").toU8a();
  await submitExtrinsic(alice, api.tx.preimage.notePreimage(remarkCallBytes), {});

  const submitProposal = api.tx.delegatedReferenda.submit(
    { system: "Root" },
    { Lookup: { hash: PREIMAGE_HASH, len: remarkCallBytes.length } },
    { After: 5 },
  );
  await submitExtrinsic(anna, submitProposal, { assetId: RELAY_ASSET_ID });

  const placeDeposit = api.tx.delegatedReferenda.placeDecisionDeposit(0);
  await submitExtrinsic(anna, placeDeposit, { assetId: RELAY_ASSET_ID });

  const voteCall = api.tx.delegatedConvictionVoting.vote(0, {
    // Voting with relay chain tokens. We know this is true; otherwise, this call
    // would fail, given that Anna doesn't have 10^16 RegionX tokens.
    Standard: { vote: { aye: true, conviction: "None" }, balance: 10n ** 16n },
  });
  await submitExtrinsic(anna, voteCall, { assetId: RELAY_ASSET_ID });
}

module.exports = { run };
