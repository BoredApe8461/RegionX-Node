import { ApiPromise, Keyring } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { getEncodedRegionId, RegionId } from 'coretime-utils';
import assert from 'node:assert';
import { sleep, submitExtrinsic } from './common';
import { makeIsmpResponse, queryRequest } from './ismp.common';

const REGIONX_SOVEREIGN_ACCOUNT = '5Eg2fntJ27qsari4FGrGhrMqKFDRnkNSR6UshkZYBGXmSuC8';

async function transferRegionToRegionX(
  coretimeApi: ApiPromise,
  regionXApi: ApiPromise,
  sender: KeyringPair,
  regionId: RegionId
) {
  const receiverKeypair = new Keyring();
  receiverKeypair.addFromAddress(sender.address);

  const feeAssetItem = 0;
  const weightLimit = 'Unlimited';
  const reserveTransferToRegionX = coretimeApi.tx.polkadotXcm.limitedReserveTransferAssets(
    { V3: { parents: 1, interior: { X1: { Parachain: 2000 } } } }, //dest
    {
      V3: {
        parents: 0,
        interior: {
          X1: {
            AccountId32: {
              chain: 'Any',
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
            Concrete: {
              parents: 1,
              interior: 'Here',
            },
          },
          fun: {
            Fungible: 10n ** 10n,
          },
        }, // ^^ fee payment asset
        {
          id: {
            Concrete: {
              parents: 0,
              interior: { X1: { PalletInstance: 50 } },
            },
          },
          fun: {
            NonFungible: {
              Index: getEncodedRegionId(regionId, coretimeApi).toString(),
            },
          },
        },
      ],
    }, //asset
    feeAssetItem,
    weightLimit
  );
  await submitExtrinsic(sender, reserveTransferToRegionX, {});

  await sleep(5000);

  const requestRecord = regionXApi.tx.regions.requestRegionRecord(regionId);
  await submitExtrinsic(sender, requestRecord, {});

  let regions = await regionXApi.query.regions.regions.entries();
  assert.equal(regions.length, 1);
  assert.deepStrictEqual(regions[0][0].toHuman(), [regionId]);

  let region = regions[0][1].toHuman() as any;
  assert(region.owner == sender.address);
  assert(typeof region.record.Pending === 'string');

  // Check the data on the Coretime chain:
  regions = await coretimeApi.query.broker.regions.entries();
  assert.equal(regions.length, 1);
  assert.deepStrictEqual(regions[0][0].toHuman(), [regionId]);
  assert.equal((regions[0][1].toHuman() as any).owner, REGIONX_SOVEREIGN_ACCOUNT);

  // Respond to the ISMP get request:
  const request = await queryRequest(regionXApi, region.record.Pending);
  await makeIsmpResponse(regionXApi, coretimeApi, request, sender.address);

  // The record should be set after ISMP response:
  regions = await regionXApi.query.regions.regions.entries();
  region = regions[0][1].toHuman() as any;
  assert(region.owner == sender.address);
}

async function transferRegionToCoretimeChain(
  coretimeApi: ApiPromise,
  regionXApi: ApiPromise,
  sender: KeyringPair,
  regionId: RegionId
) {
  const receiverKeypair = new Keyring();
  receiverKeypair.addFromAddress(sender.address);

  const feeAssetItem = 0;
  const weightLimit = 'Unlimited';

  // Transfer the region back to the Coretime chain:
  const reserveTransferToCoretime = regionXApi.tx.polkadotXcm.limitedReserveTransferAssets(
    { V3: { parents: 1, interior: { X1: { Parachain: 1005 } } } }, // dest
    {
      V3: {
        parents: 0,
        interior: {
          X1: {
            AccountId32: {
              chain: 'Any',
              id: receiverKeypair.pairs[0].publicKey,
            },
          },
        },
      },
    }, // ^^ beneficiary
    {
      V3: [
        {
          id: {
            Concrete: {
              parents: 1,
              interior: 'Here',
            },
          },
          fun: {
            Fungible: 10n ** 10n,
          },
        }, // ^^ fee payment asset
        {
          id: {
            Concrete: {
              parents: 1,
              // chain: Rococo-Coretime, pallet: pallet_broker
              interior: { X2: [{ Parachain: 1005 }, { PalletInstance: 50 }] },
            },
          },
          fun: {
            NonFungible: {
              Index: getEncodedRegionId(regionId, regionXApi).toString(),
            },
          },
        },
      ],
    }, // ^^ asset
    feeAssetItem,
    weightLimit
  );
  await submitExtrinsic(sender, reserveTransferToCoretime, {});
  await sleep(5000);

  let regions = await regionXApi.query.regions.regions.entries();
  assert.equal(regions.length, 0);

  regions = await coretimeApi.query.broker.regions.entries();
  assert.equal(regions.length, 1);
  assert.deepStrictEqual(regions[0][0].toHuman(), [regionId]);
  assert.equal((regions[0][1].toHuman() as any).owner, sender.address);
}

export { transferRegionToRegionX, transferRegionToCoretimeChain };
