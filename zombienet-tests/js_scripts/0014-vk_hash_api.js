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
const { PROOF: FFLONK_PROOF, PUBS: FFLONK_PUBS, VK: FFLONK_VK, VKEY_HASH: FFLONK_VKEY_HASH } = require('./fflonk_data.js');
const { PROOF: GROTH16_PROOF, PUBS: GROTH16_PUBS, VK: GROTH16_VK, VKEY_HASH: GROTH16_VKEY_HASH } = require('./groth16_data.js');
const { PROOF: PLONKY2_PROOF, PUBS: PLONKY2_PUBS, VK: PLONKY2_VK, VKEY_HASH: PLONKY2_VKEY_HASH } = require('./plonky2_data.js');
const { PROOF: PROOFOFSQL_PROOF, PUBS: PROOFOFSQL_PUBS, VK: PROOFOFSQL_VK, VKEY_HASH: PROOFOFSQL_VKEY_HASH } = require('./proofofsql_data.js');
const { PROOF: RISC0_PROOF, PUBS: RISC0_PUBS, VK: RISC0_VK } = require('./risc0_data.js');
const { PROOF: ULTRAPLONK_PROOF, PUBS: ULTRAPLONK_PUBS, VK: ULTRAPLONK_VK, VKEY_HASH: ULTRAPLONK_VKEY_HASH,
    STATEMENT_HASH: ULTRAPLONK_STATEMENT_HASH } = require('./ultraplonk_data.js');

async function run(nodeName, networkInfo, _args) {
    const api = await init_api(zombie, nodeName, networkInfo);

    verifiers = [
        {
            name: "Fflonk",
            pallet: api.rpc.vk_hash.fflonk,
            vk: FFLONK_VK,
            expected_hash: FFLONK_VKEY_HASH
        },
        {
            name: "Groth16",
            pallet: api.rpc.vk_hash.groth16,
            vk: GROTH16_VK,
            expected_hash: GROTH16_VKEY_HASH
        },
        {
            name: "Plonky2",
            pallet: api.rpc.vk_hash.plonky2,
            vk: PLONKY2_VK,
            expected_hash: PLONKY2_VKEY_HASH
        },
        {
            name: "Proofofsql",
            pallet: api.rpc.vk_hash.proofofsql,
            vk: PROOFOFSQL_VK,
            expected_hash: PROOFOFSQL_VKEY_HASH
        },
        {
            name: "Risc0",
            pallet: api.rpc.vk_hash.risc0,
            vk: RISC0_VK,
            expected_hash: RISC0_VK
        },
        {
            name: "UltraPLONK",
            pallet: api.rpc.vk_hash.ultraplonk,
            vk: ULTRAPLONK_VK,
            expected_hash: ULTRAPLONK_VKEY_HASH
        },
    ];

    for (const verifier of verifiers) {
        verifier_hash = await verifier.pallet(verifier.vk);
        console.log(`##### ${verifier.name} RPC returned (hash ${verifier_hash}): `
            + JSON.stringify(verifier_hash));
        if (verifier_hash != verifier.expected_hash) {
            return ReturnCode.ErrIncorrectHash;
        }
    }

    // Any return value different from 1 is considered an error
    return ReturnCode.Ok;
}

module.exports = { run }
