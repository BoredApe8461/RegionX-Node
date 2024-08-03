import { ApiPromise, Keyring, WsProvider } from '@polkadot/api';
import assert from 'node:assert';
import {
  log,
  openHrmpChannel,
  RELAY_ASSET_ID,
  setupRelayAsset,
  sleep,
  submitExtrinsic,
  transferRelayAssetToPara,
} from '../common';
import { UNIT } from '../consts';
import { configureBroker, purchaseRegion, startSales } from '../coretime.common';
import { ismpAddParachain } from '../ismp.common';
import { REGIONX_API_TYPES, REGIONX_CUSTOM_RPC } from '../types';
import { transferRegionToRegionX } from '../xc-regions.common';

async function run(_nodeName: string, networkInfo: any, _jsArgs: any) {
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
  const bob = keyring.addFromUri('//Bob');

  const txSetRelayXcmVersion = rococoApi.tx.xcmPallet.forceDefaultXcmVersion([3]);
  const txSetCoretimeXcmVersion = coretimeApi.tx.polkadotXcm.forceDefaultXcmVersion([3]);
  log('Setting XCM version: ');
  await submitExtrinsic(alice, rococoApi.tx.sudo.sudo(txSetRelayXcmVersion), {});
  await submitExtrinsic(alice, coretimeApi.tx.sudo.sudo(txSetCoretimeXcmVersion), {});

  log('Setting up relay asset: ');
  await setupRelayAsset(regionXApi, alice, 500n * UNIT);

  log('Opening HRMP: ');
  await openHrmpChannel(alice, rococoApi, 1005, 2000);
  await openHrmpChannel(alice, rococoApi, 2000, 1005);
  log('Adding ISMP: ');
  await ismpAddParachain(alice, regionXApi);

  log('Transfering rc token to RegionX:');
  await transferRelayAssetToPara(rococoApi, alice, 1005, alice.address, 100n * UNIT);
  await transferRelayAssetToPara(rococoApi, alice, 2000, alice.address, 100n * UNIT);

  log('Configuring coretime chain:');
  await configureBroker(coretimeApi, alice);
  log('Starting sales:');
  await startSales(coretimeApi, alice);

  const regionId = await purchaseRegion(coretimeApi, alice);
  if (!regionId) throw new Error('RegionId not found');

  log('Transferring region to RegionX');
  await transferRegionToRegionX(coretimeApi, regionXApi, alice, regionId);

  const paraId = 2000;
  const orderRequirements = {
    begin: 40,
    end: 45,
    coreOccupancy: 57600, // full core
  };

  log('Creating order');
  const createOrderCall = regionXApi.tx.orders.createOrder(paraId, orderRequirements);
  await submitExtrinsic(alice, createOrderCall, {});

  const order = (await regionXApi.query.orders.orders(0)).toJSON();
  assert.deepStrictEqual(order, {
    creator: alice.address,
    paraId: 2000,
    requirements: orderRequirements,
  });

  log('Giving Bob tokens');
  const transferToBobCall = regionXApi.tx.tokens.transfer(bob.address, RELAY_ASSET_ID, 30n * UNIT);
  await submitExtrinsic(alice, regionXApi.tx.sudo.sudo(transferToBobCall), {});

  log('Bob making a contribution');
  const contributeCall = regionXApi.tx.orders.contribute(0, 10n * UNIT);
  await submitExtrinsic(bob, contributeCall, {});

  log('Alice fulfilling the order');
  const fulfillCall = regionXApi.tx.processor.fulfillOrder(0, regionId);
  await submitExtrinsic(alice, fulfillCall, {});
  // Region should be removed after making the assignment call:
  const regions = await regionXApi.query.regions.regions.entries();
  assert.equal(regions.length, 0);

  // `fulfillOrder` will make a cross-chain call to the Coretime chain to make the assignment.
  // We will wait a bit since it will take some time.
  await sleep(5000);

  const workplan = await coretimeApi.query.broker.workplan.entries();
  assert.equal(workplan.length, 1);
}

export { run };
