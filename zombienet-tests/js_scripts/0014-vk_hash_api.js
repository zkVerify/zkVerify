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

const { init_api } = require('zkv-lib')
const { VK: FFLONK_VK, VKEY_HASH: FFLONK_VKEY_HASH } = require('./fflonk_data.js');
const { VK: GROTH16_VK, VKEY_HASH: GROTH16_VKEY_HASH } = require('./groth16_data.js');
const { VK: PLONKY2_VK, VKEY_HASH: PLONKY2_VKEY_HASH } = require('./plonky2_data.js');
<<<<<<< HEAD
const { VK: RISC0_VK } = require('./risc0_v2_2_data.js');
=======
const { VK: RISC0_VK } = require('./risc0_v2_1_data.js');
const { VK: ULTRAHONK_VK, VKEY_HASH: ULTRAHONK_VKEY_HASH } = require('./ultrahonk_data.js');
>>>>>>> 227852f (Initial commit)
const { VK: ULTRAPLONK_VK, VKEY_HASH: ULTRAPLONK_VKEY_HASH } = require('./ultraplonk_data.js');
const { VK: SP1_VK } = require('./sp1_data.js');

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
            name: "Risc0",
            pallet: api.rpc.vk_hash.risc0,
            vk: RISC0_VK,
            expected_hash: RISC0_VK
        },
        {
            name: "UltraHonk",
            pallet: api.rpc.vk_hash.ultrahonk,
            vk: ULTRAHONK_VK,
            expected_hash: ULTRAHONK_VKEY_HASH
        },
        {
            name: "UltraPLONK",
            pallet: api.rpc.vk_hash.ultraplonk,
            vk: ULTRAPLONK_VK,
            expected_hash: ULTRAPLONK_VKEY_HASH
        },
        {
            name: "Sp1",
            pallet: api.rpc.vk_hash.sp1,
            vk: SP1_VK,
            expected_hash: SP1_VK
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
