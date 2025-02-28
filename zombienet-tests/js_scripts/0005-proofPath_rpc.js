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
};

const { init_api, BLOCK_TIME, submitProof, waitForNewAttestation, receivedEvents } = require('zkv-lib');
const { PROOF: ZKSYNC_PROOF, PUBS: ZKSYNC_PUBS } = require('./zksync_data.js');
const { PROOF: FFLONK_PROOF, PUBS: FFLONK_PUBS, VK: FFLONK_VK } = require('./fflonk_data.js');
const { PROOF: GROTH16_PROOF, PUBS: GROTH16_PUBS, VK: GROTH16_VK } = require('./groth16_data.js');
const { PROOF: RISC0_V1_0_PROOF, PUBS: RISC0_V1_0_PUBS, VK: RISC0_V1_0_VK } = require('./risc0_data.js');
const { PROOF: RISC0_V1_2_PROOF, PUBS: RISC0_V1_2_PUBS, VK: RISC0_V1_2_VK } = require('./risc0_v1_2_data.js')
const { PROOF: ULTRAPLONK_PROOF, PUBS: ULTRAPLONK_PUBS, VK: ULTRAPLONK_VK } = require('./ultraplonk_data.js');
const { PROOF: PROOFOFSQL_PROOF, PUBS: PROOFOFSQL_PUBS, VK: PROOFOFSQL_VK } = require('./proofofsql_data.js');
const { PROOF: PLONKY2_PROOF, PUBS: PLONKY2_PUBS, VK: PLONKY2_VK } = require('./plonky2_data.js');

async function run(nodeName, networkInfo, _args) {
    const api = await init_api(zombie, nodeName, networkInfo);

    // Create a keyring instance
    const keyring = new zombie.Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');

    // Create the proof submission extrinsics...
    let proofHashesArray = [];

    verifiers = [
        {
            name: "FFlonk",
            pallet: api.tx.settlementFFlonkPallet,
            args: [{ 'Vk': FFLONK_VK }, FFLONK_PROOF, FFLONK_PUBS],
        },
        {
            name: "Zksync",
            pallet: api.tx.settlementZksyncPallet,
            args: [{ 'Vk': null }, ZKSYNC_PROOF, ZKSYNC_PUBS],
        },
        {
            name: "Risc0V1.0",
            pallet: api.tx.settlementRisc0Pallet,
            args: [{ 'Vk': RISC0_V1_0_VK }, RISC0_V1_0_PROOF, RISC0_V1_0_PUBS],
        },
        {
            name: "Risc0V1.2",
            pallet: api.tx.settlementRisc0Pallet,
            args: [{ 'Vk': RISC0_V1_2_VK }, RISC0_V1_2_PROOF, RISC0_V1_2_PUBS,]
        },
        {
            name: "Groth16",
            pallet: api.tx.settlementGroth16Pallet,
            args: [{ 'Vk': GROTH16_VK }, GROTH16_PROOF, GROTH16_PUBS],
        },
        {
            name: "Ultraplonk",
            pallet: api.tx.settlementUltraplonkPallet,
            args: [{ 'Vk': ULTRAPLONK_VK }, ULTRAPLONK_PROOF, ULTRAPLONK_PUBS],
        },
        {
            name: "Proofofsql",
            pallet: api.tx.settlementProofOfSqlPallet,
            args: [{ 'Vk': PROOFOFSQL_VK }, PROOFOFSQL_PROOF, PROOFOFSQL_PUBS],
        },
        {
            name: "Plonky2",
            pallet: api.tx.settlementPlonky2Pallet,
            args: [{ 'Vk': PLONKY2_VK }, PLONKY2_PROOF, PLONKY2_PUBS],
        }
    ];

    let proofIncludedTimestamp = null;
    let failed = false;
    for (const [index, verifier] of verifiers.entries()) {
        verifier.events = (await submitProof(verifier.pallet, alice, ...verifier.args)).events;
        if (index == 0) {
            proofIncludedTimestamp = Date.now();
        }
        if (receivedEvents(verifier.events)) {
            verifier.statementHash = verifier.events[0].data[0];
            proofHashesArray.push(verifier.statementHash);
        } else {
            console.log(`${verifier.name} proof submission failed`);
            failed = true;
        }
    }

    if (failed) {
        return ReturnCode.ErrProofVerificationFailed;
    }

    // Wait for the next attestation ID to be emitted
    const EXPECTED_ATT_TIMEOUT = BLOCK_TIME * 9.5;
    const EXPECTED_ATT_TIMEOUT_DELTA = BLOCK_TIME * 3;
    const interestingAttId = await waitForNewAttestation(api, EXPECTED_ATT_TIMEOUT * 2);
    const attTimestamp = Date.now();
    let publishedRoot;
    if (interestingAttId == -1) {
        console.log("Something went wrong while waiting for a new attestation");
        return ReturnCode.ErrNoAttestation;
    } else {
        publishedRoot = interestingAttId.data[1];
        console.log("A new attestation has been published: ");
        interestingAttId.data.forEach((data) => {
            console.log(`\t\t\t${data.toString()}`);
        });
    }

    // Check that the attestation was received in the expected time window
    if (attTimestamp < proofIncludedTimestamp + EXPECTED_ATT_TIMEOUT ||
        attTimestamp > proofIncludedTimestamp + (EXPECTED_ATT_TIMEOUT + EXPECTED_ATT_TIMEOUT_DELTA)) {
        console.log(`Attestation not received in the expected time window`);
        console.log(`${attTimestamp} < ${proofIncludedTimestamp + EXPECTED_ATT_TIMEOUT}`)
        console.log(`${attTimestamp} > ${proofIncludedTimestamp + (EXPECTED_ATT_TIMEOUT + EXPECTED_ATT_TIMEOUT_DELTA)}`);
        return ReturnCode.ErrWrongAttestationTiming;
    }

    // For each proof, get its Merkle path and evaluate the root
    const attId = parseInt(interestingAttId.data['id']);
    for (verifier of verifiers) {
        verifier.path = await api.rpc.poe.proofPath(attId, verifier.statementHash);
        console.log(`##### proofPath RPC returned (proof ${verifier.name}): ` + JSON.stringify(verifier.path));
        let checked = await verifyProof(verifier.path, publishedRoot);
        console.log(`Proof ${verifier.name} checked: ${checked}`);
        failed |= !checked;
    }

    if (failed) {
        return ReturnCode.ErrAttProofFailedVerification;
    }

    // Any return value different from 1 is considered an error
    return ReturnCode.Ok;
}

function stripHexPrefix(input_str) {
    return input_str.toString().replace(/^0x/, '');
}

function verifyProof(proof, publishedRoot) {
    let position = parseInt(proof['leaf_index'], 10);
    let width = parseInt(proof['number_of_leaves'], 10);
    let hash = Keccak256(proof['leaf'].toString('hex')).toString('hex');
    proof['proof'].forEach(function (p) {
        p = stripHexPrefix(p);
        if (position % 2 == 1 || position + 1 == width) {
            hash = Keccak256('0x' + p + hash).toString('hex');
        } else {
            hash = Keccak256('0x' + hash + p).toString('hex');
        }
        position = parseInt(Math.floor(position / 2), 10);
        width = parseInt(Math.floor((width - 1) / 2) + 1, 10);
    });

    return stripHexPrefix(publishedRoot) == hash;
}

module.exports = { run }

