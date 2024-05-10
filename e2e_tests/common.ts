import { ApiPromise, Keyring } from '@polkadot/api';
import { SignerOptions, SubmittableExtrinsic } from '@polkadot/api/types';
import { KeyringPair } from '@polkadot/keyring/types';

const RELAY_ASSET_ID = 1;

async function submitExtrinsic(
  signer: KeyringPair,
  call: SubmittableExtrinsic<'promise'>,
  options: Partial<SignerOptions>
): Promise<void> {
  return new Promise((resolve, reject) => {
    const unsub = call.signAndSend(signer, options, (result) => {
      console.log(`Current status is ${result.status}`);
      if (result.status.isInBlock) {
        console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
      } else if (result.status.isFinalized) {
        console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
        unsub.then();
        return resolve();
      } else if (result.isError) {
        console.log('Transaction error');
        unsub.then();
        return reject();
      }
    });
  });
}

async function setupRelayAsset(api: ApiPromise, signer: KeyringPair, initialBalance = 0n) {
  const assetMetadata = {
    decimals: 12,
    name: 'ROC',
    symbol: 'ROC',
    existentialDeposit: 10n ** 3n,
    location: null,
    additional: null,
  };

  const assetSetupCalls = [
    api.tx.assetRegistry.registerAsset(assetMetadata, RELAY_ASSET_ID),
    api.tx.assetRate.create(RELAY_ASSET_ID, 1_000_000_000_000_000_000n), // 1 on 1
  ];

  if (initialBalance > BigInt(0)) {
    assetSetupCalls.push(
      api.tx.tokens.setBalance(signer.address, RELAY_ASSET_ID, initialBalance, 0)
    );
  }

  const batchCall = api.tx.utility.batch(assetSetupCalls);
  const sudoCall = api.tx.sudo.sudo(batchCall);

  await submitExtrinsic(signer, sudoCall, {});
}

// Transfer the relay chain asset to the parachain specified by paraId.
// Receiver address is same as the sender's.
async function transferRelayAssetToPara(
  amount: bigint,
  paraId: number,
  relayApi: ApiPromise,
  signer: KeyringPair
) {
  const receiverKeypair = new Keyring();
  receiverKeypair.addFromAddress(signer.address);

  // If system parachain we use teleportation, otherwise we do a reserve transfer.
  const transferKind = paraId < 2000 ? 'limitedTeleportAssets' : 'limitedReserveTransferAssets';

  const feeAssetItem = 0;
  const weightLimit = 'Unlimited';
  const reserveTransfer = relayApi.tx.xcmPallet[transferKind](
    { V3: { parents: 0, interior: { X1: { Parachain: paraId } } } }, //dest
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
            Concrete: { parents: 0, interior: 'Here' },
          },
          fun: {
            Fungible: amount,
          },
        },
      ],
    }, //asset
    feeAssetItem,
    weightLimit
  );
  await submitExtrinsic(signer, reserveTransfer, {});
}

async function sleep(milliseconds: number) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

export { RELAY_ASSET_ID, setupRelayAsset, sleep, submitExtrinsic, transferRelayAssetToPara };
