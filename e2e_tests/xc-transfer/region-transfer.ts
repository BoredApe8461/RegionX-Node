import { ApiPromise, Keyring, WsProvider } from '@polkadot/api';
import {
  openHrmpChannel,
  setupRelayAsset,
  submitExtrinsic,
  transferRelayAssetToPara,
} from '../common';
import { UNIT } from '../consts';
import { configureBroker, purchaseRegion, startSales } from '../coretime.common';
import { ismpAddParachain } from '../ismp.common';
import { REGIONX_API_TYPES, REGIONX_CUSTOM_RPC } from '../types';
import { transferRegionToCoretimeChain, transferRegionToRegionX } from '../xc-regions.common';

async function run(_nodeName: any, networkInfo: any, _jsArgs: any) {
  const { wsUri: regionXUri } = networkInfo.nodesByName['regionx-collator01'];
  const { wsUri: coretimeUri } = networkInfo.nodesByName['coretime-collator01'];
  const { wsUri: rococoUri } = networkInfo.nodesByName['rococo-validator01'];

  const regionXApi = await ApiPromise.create({
    provider: new WsProvider(regionXUri),
    types: { ...REGIONX_API_TYPES },
    rpc: REGIONX_CUSTOM_RPC,
  });
  const rococoApi = await ApiPromise.create({ provider: new WsProvider(rococoUri) });
  const coretimeApi = await ApiPromise.create({ provider: new WsProvider(coretimeUri) });

  // account to submit tx
  const keyring = new Keyring({ type: 'sr25519' });
  const alice = keyring.addFromUri('//Alice');

  const txSetCoretimeXcmVersion = coretimeApi.tx.polkadotXcm.forceDefaultXcmVersion([3]);
  const txSetRelayXcmVersion = rococoApi.tx.xcmPallet.forceDefaultXcmVersion([3]);
  await submitExtrinsic(alice, coretimeApi.tx.sudo.sudo(txSetCoretimeXcmVersion), {});
  await submitExtrinsic(alice, rococoApi.tx.sudo.sudo(txSetRelayXcmVersion), {});

  await setupRelayAsset(regionXApi, alice);

  await openHrmpChannel(alice, rococoApi, 1005, 2000);
  await openHrmpChannel(alice, rococoApi, 2000, 1005);
  await ismpAddParachain(alice, regionXApi);

  await transferRelayAssetToPara(rococoApi, alice, 1005, alice.address, 1000n * UNIT);
  await transferRelayAssetToPara(rococoApi, alice, 2000, alice.address, 1000n * UNIT);

  await configureBroker(coretimeApi, alice);
  await startSales(coretimeApi, alice);

  const regionId = await purchaseRegion(coretimeApi, alice);
  if (!regionId) throw new Error('RegionId not found');

  // Transferring to the RegionX chain should work:
  // NOTE: the function contains checks, and if any of them fail, the test will fail.
  await transferRegionToRegionX(coretimeApi, regionXApi, alice, regionId);

  // Transferring back to the Coretime chain should work:
  // NOTE: the function contains checks, and if any of them fail, the test will fail.
  await transferRegionToCoretimeChain(coretimeApi, regionXApi, alice, regionId);
}

export { run };
