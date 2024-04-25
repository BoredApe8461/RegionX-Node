async function submitExtrinsic(signer, call, options) {
  return new Promise(async (resolve, reject) => {
    const unsub = await call.signAndSend(signer, options, (result) => {
      console.log(`Current status is ${result.status}`);
      if (result.status.isInBlock) {
        console.log(
          `Transaction included at blockHash ${result.status.asInBlock}`
        );
      } else if (result.status.isFinalized) {
        console.log(
          `Transaction finalized at blockHash ${result.status.asFinalized}`
        );
        unsub();
        return resolve();
      } else if (result.isError) {
        console.log(`Transaction error`);
        unsub();
        return resolve();
      }
    });
  });
}

module.exports = { submitExtrinsic }
