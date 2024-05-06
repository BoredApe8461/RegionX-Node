import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";
import { submitExtrinsic, setupRelayAsset, RELAY_ASSET_ID } from "../common";

async function run(nodeName: string, networkInfo: any, _jsArgs: any) {
	const { wsUri: regionXUri } = networkInfo.nodesByName[nodeName];
	const { wsUri: rococoUri } = networkInfo.nodesByName["rococo-validator01"];

	const rococoApi = await ApiPromise.create({ provider: new WsProvider(rococoUri) });
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
	const keyring = new Keyring({ type: "sr25519" });
	const alice = keyring.addFromUri("//Alice");

	const setXcmVersion = rococoApi.tx.xcmPallet.forceDefaultXcmVersion([3]);
	await submitExtrinsic(alice, rococoApi.tx.sudo.sudo(setXcmVersion), {});

	await setupRelayAsset(regionXApi, alice);

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
		weightLimit
	);
	await submitExtrinsic(alice, reserveTransfer, {});

	// Try to pay for fees with relay chain asset.
	const remarkCall = regionXApi.tx.system.remark("0x44");
	await submitExtrinsic(alice, remarkCall, { assetId: RELAY_ASSET_ID });
}

export { run };
