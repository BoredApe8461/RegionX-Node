import { ApiPromise } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { ISubmittableResult } from '@polkadot/types/types';
import { submitExtrinsic } from './common';
import { Get, IsmpRequest } from './types';

async function ismpAddParachain(signer: KeyringPair, regionXApi: ApiPromise) {
  const addParaCall = regionXApi.tx.ismpParachain.addParachain([1005]);
  const sudoCall = regionXApi.tx.sudo.sudo(addParaCall);
  return submitExtrinsic(signer, sudoCall, {});
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

export { makeIsmpResponse, queryRequest, ismpAddParachain };
