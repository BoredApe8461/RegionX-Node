import { ApiPromise, Keyring, WsProvider } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { getEncodedRegionId, Id, RegionId } from 'coretime-utils';
import assert from 'node:assert';
import { setupRelayAsset, sleep, submitExtrinsic, transferRelayAssetToPara } from '../common';
import { CONFIG, CORE_COUNT, INITIAL_PRICE, UNIT } from '../consts';

const REGIONX_SOVEREIGN_ACCOUNT = '5Eg2fntJ27qsari4FGrGhrMqKFDRnkNSR6UshkZYBGXmSuC8';

async function run(_nodeName: any, networkInfo: any, _jsArgs: any) {
  const { wsUri: regionXUri } = networkInfo.nodesByName['regionx-collator01'];
  const { wsUri: coretimeUri } = networkInfo.nodesByName['coretime-collator01'];
  const { wsUri: rococoUri } = networkInfo.nodesByName['rococo-validator01'];

  const regionXApi = await ApiPromise.create({ provider: new WsProvider(regionXUri) });
  const rococoApi = await ApiPromise.create({
    provider: new WsProvider(rococoUri),
    types: { Id },
  });
  const coretimeApi = await ApiPromise.create({
    provider: new WsProvider(coretimeUri),
    types: { Id },
  });

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

  // Needed for fee payment
  // The Coretime chain account by default has tokens for fee payment.
  await transferRelayAssetToPara(10n ** 12n, 2000, rococoApi, alice);

  await configureBroker(coretimeApi, alice);
  await startSales(coretimeApi, alice);

  const txSetBalance = coretimeApi.tx.balances.forceSetBalance(alice.address, 1000 * UNIT);
  await submitExtrinsic(alice, coretimeApi.tx.sudo.sudo(txSetBalance), {});

  await ismpAddParachain(alice, regionXApi);

  const regionId = await purchaseRegion(coretimeApi, alice);
  if (!regionId) throw new Error('RegionId not found');

  const receiverKeypair = new Keyring();
  receiverKeypair.addFromAddress(alice.address);

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
  await submitExtrinsic(alice, reserveTransferToRegionX, {});

  await sleep(5000);

  let regions = await regionXApi.query.regions.regions.entries();
  assert.equal(regions.length, 1);
  assert.deepStrictEqual(regions[0][0].toHuman(), [regionId]);
  // record is unavailable because we did not setup ismp.
  assert((regions[0][1].toHuman() as any).owner == alice.address);
  assert(typeof (regions[0][1].toHuman() as any).record.Pending === 'string');

  regions = await coretimeApi.query.broker.regions.entries();
  assert.equal(regions.length, 1);
  assert.deepStrictEqual(regions[0][0].toHuman(), [regionId]);
  assert.equal((regions[0][1].toHuman() as any).owner, REGIONX_SOVEREIGN_ACCOUNT);

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
  await submitExtrinsic(alice, reserveTransferToCoretime, {});
  await sleep(5000);

  regions = await regionXApi.query.regions.regions.entries();
  assert.equal(regions.length, 0);

  regions = await coretimeApi.query.broker.regions.entries();
  assert.equal(regions.length, 1);
  assert.deepStrictEqual(regions[0][0].toHuman(), [regionId]);
  assert.equal((regions[0][1].toHuman() as any).owner, alice.address);
}

async function ismpAddParachain(signer: KeyringPair, regionXApi: ApiPromise) {
  const addParaCall = regionXApi.tx.ismpParachain.addParachain([1005]);
  const sudoCall = regionXApi.tx.sudo.sudo(addParaCall);
  return submitExtrinsic(signer, sudoCall, {});
}

async function openHrmpChannel(
  signer: KeyringPair,
  relayApi: ApiPromise,
  senderParaId: number,
  recipientParaId: number
) {
  const openHrmp = relayApi.tx.parasSudoWrapper.sudoEstablishHrmpChannel(
    senderParaId, // sender
    recipientParaId, // recipient
    8, // Max capacity
    102400 // Max message size
  );
  const sudoCall = relayApi.tx.sudo.sudo(openHrmp);

  return submitExtrinsic(signer, sudoCall, {});
}

async function configureBroker(coretimeApi: ApiPromise, signer: KeyringPair): Promise<void> {
  const configCall = coretimeApi.tx.broker.configure(CONFIG);
  const sudo = coretimeApi.tx.sudo.sudo(configCall);
  return submitExtrinsic(signer, sudo, {});
}

async function startSales(coretimeApi: ApiPromise, signer: KeyringPair): Promise<void> {
  const startSaleCall = coretimeApi.tx.broker.startSales(INITIAL_PRICE, CORE_COUNT);
  const sudo = coretimeApi.tx.sudo.sudo(startSaleCall);
  return submitExtrinsic(signer, sudo, {});
}

async function purchaseRegion(
  coretimeApi: ApiPromise,
  buyer: KeyringPair
): Promise<RegionId | null> {
  const callTx = async (resolve: (regionId: RegionId | null) => void) => {
    const purchase = coretimeApi.tx.broker.purchase(INITIAL_PRICE * 2);
    const unsub = await purchase.signAndSend(buyer, async (result: any) => {
      if (result.status.isInBlock) {
        const regionId = await getRegionId(coretimeApi);
        unsub();
        resolve(regionId);
      }
    });
  };

  return new Promise(callTx);
}

async function getRegionId(coretimeApi: ApiPromise): Promise<RegionId | null> {
  const events: any = await coretimeApi.query.system.events();

  for (const record of events) {
    const { event } = record;
    if (event.section === 'broker' && event.method === 'Purchased') {
      const data = event.data[1].toHuman();
      return data;
    }
  }

  return null;
}

export { run };
