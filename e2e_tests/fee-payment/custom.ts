import { ApiPromise, Keyring, WsProvider } from '@polkadot/api';
import { RELAY_ASSET_ID, setupRelayAsset, submitExtrinsic } from '../common';

async function run(nodeName: string, networkInfo: any, _jsArgs: any) {
  const { wsUri: regionXUri } = networkInfo.nodesByName[nodeName];
  const { wsUri: rococoUri } = networkInfo.nodesByName['rococo-validator01'];

  const rococoApi = await ApiPromise.create({ provider: new WsProvider(rococoUri) });
  const regionXApi = await ApiPromise.create({
    provider: new WsProvider(regionXUri),
    signedExtensions: {
      ChargeAssetTxPayment: {
        extrinsic: {
          tip: 'Compact<Balance>',
          assetId: 'Option<AssetId>',
        },
        payload: {},
      },
    },
  });

  // account to submit tx
  const keyring = new Keyring({ type: 'sr25519' });
  const alice = keyring.addFromUri('//Alice');

  const setXcmVersion = rococoApi.tx.xcmPallet.forceDefaultXcmVersion([3]);
  await submitExtrinsic(alice, rococoApi.tx.sudo.sudo(setXcmVersion), {});

  await setupRelayAsset(regionXApi, alice, 10n ** 12n);

  const receiverKeypair = new Keyring();
  receiverKeypair.addFromAddress(alice.address);

  // Try to pay for fees with relay chain asset.
  const remarkCall = regionXApi.tx.system.remark('0x44');
  await submitExtrinsic(alice, remarkCall, { assetId: RELAY_ASSET_ID });
}

export { run };
