import { ApiPromise, Keyring, WsProvider } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { ISubmittableResult } from '@polkadot/types/types';
import { getEncodedRegionId, RegionId } from 'coretime-utils';
import assert from 'node:assert';
import { setupRelayAsset, sleep, submitExtrinsic, transferRelayAssetToPara } from '../common';
import { CONFIG, CORE_COUNT, INITIAL_PRICE, UNIT } from '../consts';
import { Get, IsmpRequest, REGIONX_API_TYPES, REGIONX_CUSTOM_RPC } from './types';

const REGIONX_SOVEREIGN_ACCOUNT = '5Eg2fntJ27qsari4FGrGhrMqKFDRnkNSR6UshkZYBGXmSuC8';

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

  let region = regions[0][1].toHuman() as any;
  assert(region.owner == alice.address);
  assert(typeof region.record.Pending === 'string');

  // Check the data on the Coretime chain:
  regions = await coretimeApi.query.broker.regions.entries();
  assert.equal(regions.length, 1);
  assert.deepStrictEqual(regions[0][0].toHuman(), [regionId]);
  assert.equal((regions[0][1].toHuman() as any).owner, REGIONX_SOVEREIGN_ACCOUNT);

  // Respond to the ISMP get request:
  const request = await queryRequest(regionXApi, region.record.Pending);
  await makeIsmpResponse(regionXApi, coretimeApi, request, alice.address);

  // The record should be set after ISMP response:
  regions = await regionXApi.query.regions.regions.entries();
  region = regions[0][1].toHuman() as any;
  assert(region.owner == alice.address);
  assert.deepStrictEqual(region.record.Available, {
    end: '66',
    owner: '5C6cBwdHw3agsBKzjGABaMq1kgXnmKyaBPN8J6c8MkHBnKu5',
    paid: null,
  });

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

async function queryRequest(regionxApi: ApiPromise, commitment: string): Promise<IsmpRequest> {
  const leafIndex = regionxApi.createType('LeafIndexQuery', { commitment });
  const requests = await (regionxApi as any).rpc.ismp.queryRequests([leafIndex]);
  // We only requested a single request so we only get one in the response.
  return requests.toJSON()[0] as IsmpRequest;
}

async function makeIsmpResponse(
  regionXApi: ApiPromise,
  coretimeApi: ApiPromise,
  request: IsmpRequest,
  responderAddress: string
): Promise<void> {
  if (isGetRequest(request)) {
    const hashAt = (
      await coretimeApi.query.system.blockHash(Number(request.get.height))
    ).toString();
    const proofData = await coretimeApi.rpc.state.getReadProof([request.get.keys[0]], hashAt);

    const stateMachineProof = regionXApi.createType('StateMachineProof', {
      hasher: 'Blake2',
      storage_proof: proofData.proof,
    });

    const substrateStateProof = regionXApi.createType('SubstrateStateProof', {
      StateProof: stateMachineProof,
    });
    const response = regionXApi.tx.ismp.handleUnsigned([
      {
        Response: {
          datagram: {
            Request: [request],
          },
          proof: {
            height: {
              id: {
                stateId: {
                  Kusama: 1005,
                },
                consensusStateId: 'PARA',
              },
              height: request.get.height.toString(),
            },
            proof: substrateStateProof.toHex(),
          },
          signer: responderAddress,
        },
      },
    ]);

    return new Promise((resolve, reject) => {
      const unsub = response.send((result: ISubmittableResult) => {
        const { status, isError } = result;
        console.log(`Current status is ${status}`);
        if (status.isInBlock) {
          console.log(`Transaction included at blockHash ${status.asInBlock}`);
        } else if (status.isFinalized) {
          console.log(`Transaction finalized at blockHash ${status.asFinalized}`);
          unsub.then();
          return resolve();
        } else if (isError) {
          console.log('Transaction error');
          unsub.then();
          return reject();
        }
      });
    });
  } else {
    new Error('Expected a Get request');
  }
}

const isGetRequest = (request: IsmpRequest): request is { get: Get } => {
  return (request as { get: Get }).get !== undefined;
};

export { run };
