import { ApiPromise, Keyring, WsProvider } from '@polkadot/api';
import assert from 'node:assert';
import { getAddressFromModuleId, getFreeBalance } from '../common';

async function run(nodeName: string, networkInfo: any, _jsArgs: any) {
  const { wsUri: regionXUri } = networkInfo.nodesByName[nodeName];
  const regionXApi = await ApiPromise.create({ provider: new WsProvider(regionXUri) });

  // account to submit tx
  const keyring = new Keyring({ type: 'sr25519' });
  const alice = keyring.addFromUri('//Alice');

  const treasuryId = regionXApi.consts.treasury.palletId.toHuman() as string;
  const treasuryAccount = getAddressFromModuleId(treasuryId);

  const podId = 'PotStake'; // FIXME: remove this hard-coded constant and fetch the on-chain value.
  const potAccount = getAddressFromModuleId(podId);

  const treasuryBalanceOld = await getFreeBalance(regionXApi, treasuryAccount);
  const potBalanceOld = await getFreeBalance(regionXApi, potAccount);

  const call = regionXApi.tx.system.remark('0x44');

  let fee = BigInt(0),
    tips = BigInt(0);

  const promise: Promise<void> = new Promise((resolve, reject) => {
    const unsub = call.signAndSend(alice, { tip: 1_000_000_000 }, ({ status, isError, events }) => {
      console.log(`Current status is ${status}`);
      if (status.isInBlock) {
        console.log(`Transaction included at blockHash ${status.asInBlock}`);
      } else if (status.isFinalized) {
        console.log(`Transaction finalized at blockHash ${status.asFinalized}`);
        for (const event of events) {
          const {
            event: { data, method, section },
          } = event;
          if (section === 'transactionPayment' && method === 'TransactionFeePaid') {
            const args = data.toJSON() as [string, number, number];
            tips = BigInt(args[2]);
            fee = BigInt(args[1]) - tips;
          }
        }

        unsub.then();
        return resolve();
      } else if (isError) {
        console.log('Transaction error');
        unsub.then();
        return reject();
      }
    });
  });
  await promise;
  const treasuryBalanceNew = await getFreeBalance(regionXApi, treasuryAccount);
  const potBalanceNew = await getFreeBalance(regionXApi, potAccount);

  const fee2Treasury = (fee * 60n) / 100n;
  const fee2Collators = fee - fee2Treasury + tips;

  assert.equal(treasuryBalanceNew - treasuryBalanceOld, fee2Treasury);
  assert.equal(potBalanceNew - potBalanceOld, fee2Collators);
}

export { run };
