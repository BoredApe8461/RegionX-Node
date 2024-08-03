import { ApiPromise } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { RegionId } from 'coretime-utils';
import { submitExtrinsic } from './common';
import { CONFIG, CORE_COUNT, INITIAL_PRICE } from './consts';

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
    const purchase = coretimeApi.tx.broker.purchase(INITIAL_PRICE * 10n);
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

export { configureBroker, startSales, purchaseRegion };
