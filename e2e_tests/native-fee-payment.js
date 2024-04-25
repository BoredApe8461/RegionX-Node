const { ApiPromise, WsProvider } = require("@polkadot/api");
const { submitExtrinsic } = require("./common");

async function run(nodeName, networkInfo, _jsArgs) {
  const { wsUri } = networkInfo.nodesByName[nodeName];
  const api = await ApiPromise.create({provider: new WsProvider(wsUri)});

  // account to submit tx
  const keyring = new zombie.Keyring({ type: "sr25519" });
  const alice = keyring.addFromUri("//Alice");
  const bob = keyring.addFromUri("//Bob");

  const call = api.tx.balances.transferKeepAlive(bob.address, 10n**6n);
  const sudo = api.tx.sudo.sudo(call);
  await submitExtrinsic(alice, sudo, {});
}

module.exports = { run };
