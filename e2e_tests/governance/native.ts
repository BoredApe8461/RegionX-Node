import { ApiPromise, Keyring, WsProvider } from '@polkadot/api';
import { setupRelayAsset, submitExtrinsic } from '../common';

const PREIMAGE_HASH = '0x0ccf4369e9a9f88f035828ba0dd5da645d5c0fa7baa86bdc8d7a80c183ab84c9';

async function run(nodeName: string, networkInfo: any, _jsArgs: any) {
  const { wsUri } = networkInfo.nodesByName[nodeName];
  const api = await ApiPromise.create({ provider: new WsProvider(wsUri) });

  // account to submit tx
  const keyring = new Keyring({ type: 'sr25519' });
  const alice = keyring.addFromUri('//Alice');

  // relay asset is needed for storing the preimage.
  await setupRelayAsset(api, alice);

  const spendCallBytes = api.tx.treasury.spendLocal(10n ** 6n, alice.address).toU8a();
  await submitExtrinsic(alice, api.tx.preimage.notePreimage(spendCallBytes), {});

  const submitProposal = api.tx.nativeReferenda.submit(
    { Origins: 'SmallTipper' },
    { Lookup: { hash: PREIMAGE_HASH, len: spendCallBytes.length } },
    { After: 5 }
  );
  await submitExtrinsic(alice, submitProposal, {});

  const placeDeposit = api.tx.nativeReferenda.placeDecisionDeposit(0);
  await submitExtrinsic(alice, placeDeposit, {});

  const voteCall = api.tx.nativeConvictionVoting.vote(0, {
    Standard: { vote: { aye: true, conviction: 'None' }, balance: 10n ** 16n },
  });
  await submitExtrinsic(alice, voteCall, {});
}

export { run };
