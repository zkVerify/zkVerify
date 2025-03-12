// This script is used to test the proofPath RPC call.
// It also shows how to properly register custom data types and RPC calls
// to Polkadot.js, in order to use its interface to interact with the blockchain.
// Finally, it also demonstrate:
// - how to submit an extrinsic and wait for its inclusion in a block
// - how to wait for a specific event to be emitted
// Both operations are performed through the use of polkadot.js observer pattern
// and promise-based async/await syntax.

const Keccak256 = require('keccak256')

const ReturnCode = {
    Ok: 1,
    ErrProofVerificationFailed: 2,
    ErrNoAttestation: 3,
    ErrAttProofVerificationFailed: 4,
    ErrWrongAttestationTiming: 5,
    ErrIncorrectHash: 6,
};

const { init_api, submitProof, registerVk, receivedEvents } = require('zkv-lib')
const { PROOF: ULTRAPLONK_PROOF, PUBS: ULTRAPLONK_PUBS, VK: VK_ULTRAPLONK, VKEY_HASH: ULTRAPLONK_VKEY_HASH,
    STATEMENT_HASH: ULTRAPLONK_STATEMENT_HASH } = require('./ultraplonk_data.js');
const { PROOF: PROOFOFSQL_PROOF, PUBS: PROOFOFSQL_PUBS, VK: VK_PROOFOFSQL, VKEY_HASH: PROOFOFSQL_VKEY_HASH } = require('./proofofsql_data.js');

async function run(nodeName, networkInfo, _args) {
    const api = await init_api(zombie, nodeName, networkInfo);

    verifier_hash = await api.rpc.compute.ultraplonk(VK_ULTRAPLONK);
    console.log(`##### UltraPLONK RPC returned (hash ${verifier_hash}): ` + JSON.stringify(verifier_hash));

    if (verifier_hash != ULTRAPLONK_VKEY_HASH) {
        return ReturnCode.ErrIncorrectHash;
    }

    verifier_hash = await api.rpc.compute.proofofsql(VK_PROOFOFSQL);
    console.log(`##### ProofOfSQL RPC returned (hash ${verifier_hash}): ` + JSON.stringify(verifier_hash));

    if (verifier_hash != PROOFOFSQL_VKEY_HASH) {
        return ReturnCode.ErrIncorrectHash;
    }

    // Any return value different from 1 is considered an error
    return ReturnCode.Ok;
}

module.exports = { run }
