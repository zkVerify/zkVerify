// This script is used to test the statementPath RPC call and register/unregister_domain extrinsic.

const Keccak256 = require('keccak256')

const ReturnCode = {
    Ok: 1,
    ErrDomainRegistrationFailed: 2,
    ErrProofVerificationFailed: 3,
    ErrAggregationNotReady: 4,
    ErrNoAggregation: 5,
    ErrAggregationProofFailedVerification: 6,
    ErrDomainUnregistrationFailed: 7,
    ErrDomainHoldFailed: 8,
    ErrProofOnUnregisteredDomain: 9,
    ErrWrongDomainId: 10,
    ErrPublishShouldNotPay: 11,
    ErrProofFromInvalidSubmitter: 12,
    ErrProofFromValidSubmitter: 13,
    ErrAllowlist: 14,
};

const { init_api, submitProof, receivedEvents, registerDomain, sudoRegisterDomain,
    holdDomain, unregisterDomain, allowlistProofSubmitters, removeProofSubmitters, aggregate, getBalance, isVolta
} = require('zkv-lib');
const { PROOF: EZKL_PROOF, PUBS: EZKL_PUBS, VK: EZKL_VK } = require('./ezkl_data.js');
const { PROOF: FFLONK_PROOF, PUBS: FFLONK_PUBS, VK: FFLONK_VK } = require('./fflonk_data.js');
const { PROOF: GROTH16_PROOF, PUBS: GROTH16_PUBS, VK: GROTH16_VK } = require('./groth16_data.js');
const { PROOF: RISC0_V2_2_PROOF, PUBS: RISC0_V2_2_PUBS, VK: RISC0_V2_2_VK } = require('./risc0_v2_2_data.js');
const { PROOF: RISC0_V3_0_PROOF, PUBS: RISC0_V3_0_PUBS, VK: RISC0_V3_0_VK } = require('./risc0_v3_0_data.js');
const { ZK_PROOF: ULTRAHONK_ZK_PROOF, PLAIN_PROOF: ULTRAHONK_PLAIN_PROOF, PUBS: ULTRAHONK_PUBS, VK: ULTRAHONK_VK } = require('./ultrahonk_data.js');
const { PROOF: ULTRAPLONK_PROOF, PUBS: ULTRAPLONK_PUBS, VK: ULTRAPLONK_VK } = require('./ultraplonk_data.js');
const { PROOF: PLONKY2_PROOF, PUBS: PLONKY2_PUBS, VK: PLONKY2_VK } = require('./plonky2_data.js');
const { PROOF: SP1_PROOF, PUBS: SP1_PUBS, VK: SP1_VK } = require('./sp1_data.js');
const { PROOF: TEE_PROOF, PUBS: TEE_PUBS, VK_TCB_RESP: TEE_VK_TCB_RESP , VK_TCB_CERT: TEE_VK_TCB_CERT } = require('./tee_intel_data.js');

async function run(nodeName, networkInfo, _args) {
    const api = await init_api(zombie, nodeName, networkInfo);

    // Create a keyring instance
    const keyring = new zombie.Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');
    const bob = keyring.addFromUri('//Bob');

    // Create the proof submission extrinsics...
    let proofHashesArray = [];
    const destination = { None: null };

    verifiers = [
        {
            name: "FFlonk",
            pallet: api.tx.settlementFFlonkPallet,
            args: [{ 'Vk': FFLONK_VK }, FFLONK_PROOF, FFLONK_PUBS],
        },
        {
            name: "Risc0.V.2.2",
            pallet: api.tx.settlementRisc0Pallet,
            args: [{ 'Vk': RISC0_V2_2_VK }, RISC0_V2_2_PROOF, RISC0_V2_2_PUBS],
        },
        {
            name: "Risc0.V.3.0",
            pallet: api.tx.settlementRisc0Pallet,
            args: [{ 'Vk': RISC0_V3_0_VK }, RISC0_V3_0_PROOF, RISC0_V3_0_PUBS],
        },
        {
            name: "Groth16",
            pallet: api.tx.settlementGroth16Pallet,
            args: [{ 'Vk': GROTH16_VK }, GROTH16_PROOF, GROTH16_PUBS],
        },
        {
            name: "Ultrahonk (ZK)",
            pallet: api.tx.settlementUltrahonkPallet,
            args: [{ 'Vk': ULTRAHONK_VK }, ULTRAHONK_ZK_PROOF, ULTRAHONK_PUBS],
        },
        {
            name: "Ultrahonk (Plain)",
            pallet: api.tx.settlementUltrahonkPallet,
            args: [{ 'Vk': ULTRAHONK_VK }, ULTRAHONK_PLAIN_PROOF, ULTRAHONK_PUBS],
        },
        {
            name: "Ultraplonk",
            pallet: api.tx.settlementUltraplonkPallet,
            args: [{ 'Vk': ULTRAPLONK_VK }, ULTRAPLONK_PROOF, ULTRAPLONK_PUBS],
        },
        {
            name: "Plonky2",
            pallet: api.tx.settlementPlonky2Pallet,
            args: [{ 'Vk': PLONKY2_VK }, PLONKY2_PROOF, PLONKY2_PUBS],
        },
        {
            name: "Sp1",
            pallet: api.tx.settlementSp1Pallet,
            args: [{ 'Vk': SP1_VK }, SP1_PROOF, SP1_PUBS],
        },
        {
            name: "Tee",
            pallet: api.tx.settlementTeePallet,
            args: [{ 'Vk': [ TEE_VK_TCB_RESP, TEE_VK_TCB_CERT ] }, TEE_PROOF, TEE_PUBS],
        }
    ];

    // Verifiers exclusive to Volta.
    if (await isVolta(api)) {
        verifiers.push({
            name: "Ezkl",
            pallet: api.tx.settlementEzklPallet,
            args: [{ 'Vk': EZKL_VK }, EZKL_PROOF, EZKL_PUBS],
        });
    }

    events = await registerDomain(bob, verifiers.length, null, "Untrusted", "Untrusted", destination, null);
    if (!receivedEvents(events)) {
        console.log(`Register Domain Error`);
        return ReturnCode.ErrDomainRegistrationFailed;
    }
    let domainId = events.events[0].data[0];
    console.log(`Domain registered successfully: ${events.events}`);

    let failed = false;
    let AggregationComplete = [];
    for (const [index, verifier] of verifiers.entries()) {
        let data = await submitProof(verifier.pallet, alice, ...verifier.args, domainId);
        verifier.data = data;
        if (receivedEvents(verifier.data)) {
            verifier.statementHash = verifier.data.events[0].data[0];
            verifier.domain_id = verifier.data.events[0].data[1];
            verifier.aggregation_id = verifier.data.events[0].data[2];
            proofHashesArray.push(verifier.statementHash);
        } else {
            console.log(`${verifier.name} proof submission failed`);
            failed = true;
        }
        AggregationComplete = AggregationComplete.concat(verifier.data.events.filter((event) => event.section === 'aggregate' && event.method === 'AggregationComplete'));
    }

    if (failed) {
        return ReturnCode.ErrProofVerificationFailed;
    }

    if (AggregationComplete.length === 0) {
        console.log(`No aggregation to aggregate`);
        return ReturnCode.ErrAggregationNotReady;
    }

    let receipts = [];

    for (const event of AggregationComplete) {
        let d_id = event.data[0];
        let id = event.data[1];
        let prePublish = await getBalance(bob);
        console.log(`Bob balance before aggregation: ${prePublish.toHuman()}`);
        let data = await aggregate(bob, d_id, id);
        if (!receivedEvents(data)) {
            console.log(`Aggregation Error`);
            return ReturnCode.ErrNoAggregation;
        }
        let aggregation = data.events.filter((event) => event.method === 'NewAggregationReceipt')[0]
        console.log(`New aggregation receipt emitted: ${aggregation}`);
        receipts.push({
            block: data.block,
            receipt: aggregation
        }
        );
        let after = await getBalance(bob);
        console.log(`Bob balance after aggregation: ${after.toHuman()}`);
        if (after <= prePublish) {
            console.log(`Bob should not pay for aggregation`);
            return ReturnCode.ErrPublishShouldNotPay;
        }
    }

    for (const { block, receipt } of receipts) {
        console.log(`receipt: ${JSON.stringify(receipt)}`);
        const domainId = receipt.data[0];
        const id = receipt.data[1];
        const publishedRoot = receipt.data[2];
        console.log(`##### block for proof ${block}`);
        for (const verifier of verifiers) {
            verifier.path = await api.rpc.aggregate.statementPath(block, domainId, id, verifier.statementHash);
            console.log(`##### statementPath RPC returned (proof ${verifier.name}): ` + JSON.stringify(verifier.path));
            let checked = await verifyProof(verifier.path, publishedRoot);
            console.log(`Proof ${verifier.name} checked: ${checked}`);
            failed |= !checked;
        }
    }

    if (failed) {
        return ReturnCode.ErrAggregationProofFailedVerification;
    }

    events = await unregisterDomain(alice, domainId);
    if (receivedEvents(events)) {
        console.log(`Unregister Domain Error: alice cannot unregister the domain`);
        return ReturnCode.ErrDomainUnregistrationFailed;
    }

    events = await holdDomain(alice, domainId);
    if (receivedEvents(events)) {
        console.log(`Hold Domain Error: alice cannot hold the domain`);
        return ReturnCode.ErrDomainHoldFailed;
    }

    events = await unregisterDomain(bob, domainId);
    if (receivedEvents(events)) {
        console.log(`Unregister Domain Error: bob cannot unregister the domain till is not on hold`);
        return ReturnCode.ErrDomainUnregistrationFailed;
    }

    let data = await holdDomain(bob, domainId);
    if (!receivedEvents(data)) {
        console.log(`Hold Domain Error: bob hold domain failed`);
        return ReturnCode.ErrDomainHoldFailed;
    }
    let state = data.events.filter((event) => event.method === 'DomainStateChanged')[0].data[1].toString();
    if (state !== "Removable") {
        console.log(`The domain should go in the 'Removable' state but we found ${state} instead`);
        return ReturnCode.ErrDomainHoldFailed;
    }

    data = await unregisterDomain(bob, domainId);
    if (!receivedEvents(data)) {
        console.log(`Unregister Domain Error: bob unregister a Removable Domain`);
        return ReturnCode.ErrDomainUnregistrationFailed;
    }
    state = data.events.filter((event) => event.method === 'DomainStateChanged')[0].data[1].toString();
    if (state !== "Removed") {
        console.log(`The domain should go in the 'Removed' state but we found ${state} instead`);
        return ReturnCode.ErrDomainHoldFailed;
    }

    data = await submitProof(verifiers[0].pallet, alice, ...verifiers[0].args, domainId);
    if (data.events.filter((event) => event.method === 'NewProof').length > 0) {
        console.log(`Accept proof on unregistered domain`);
        return ReturnCode.ErrProofOnUnregisteredDomain;
    }

    data = await registerDomain(bob, 4, 8, "Untrusted", "Untrusted", destination, null);
    if (!receivedEvents(data)) {
        console.log(`Register Domain Error`);
        return ReturnCode.ErrDomainRegistrationFailed;
    }

    console.log(`Domain registered successfully: ${data.events}`);
    let newDomainId = data.events[0].data[0];
    if (newDomainId <= domainId) {
        console.log(`Domain registered with wrong id`);
        return ReturnCode.ErrWrongDomainId;
    }

    // Now we are checking the hold state machine.
    let verifier = verifiers[0];
    data = await submitProof(verifier.pallet, alice, ...verifier.args, newDomainId);
    if (!receivedEvents(data)) {
        console.log(`Verify proof error on hold state machine`);
        return ReturnCode.ErrProofVerificationFailed;
    }
    let aggId = data.events.filter((event) => event.method === 'NewProof')[0].data[2];
    data = await holdDomain(bob, newDomainId);
    if (!receivedEvents(data)) {
        console.log(`Hold Domain Error: on verify hold state machine`);
        return ReturnCode.ErrDomainHoldFailed;
    }
    state = data.events.filter((event) => event.method === 'DomainStateChanged')[0].data[1].toString();
    if (state !== "Hold") {
        console.log(`The domain should go in the 'Hold' state but we found ${state} instead`);
        return ReturnCode.ErrDomainHoldFailed;
    }

    console.log(`Aggregating domain ${newDomainId} agg ${aggId}`);
    data = await aggregate(bob, newDomainId, aggId);
    if (!receivedEvents(data)) {
        console.log(`Aggregation Error`);
        return ReturnCode.ErrNoAggregation;
    }
    if (data.events.filter((event) => event.method === 'DomainStateChanged')[0].data[1].toString() !== "Removable") {
        console.log(`Hold Domain Error: on verify hold state machine - invalid hold state ${states}`);
        return ReturnCode.ErrDomainHoldFailed;
    }

    // Only Owner can submit
    data = await registerDomain(bob, 4, 8, "Untrusted", "OnlyOwner", destination, null);
    if (!receivedEvents(data)) {
        console.log(`Register Domain Error`);
        return ReturnCode.ErrDomainRegistrationFailed;
    }

    console.log(`Domain registered successfully: ${data.events}`);
    domainId = data.events[0].data[0];

    // Now we are checking the hold state machine.
    verifier = verifiers[0];

    // Alice cannot post proof
    data = await submitProof(verifier.pallet, alice, ...verifier.args, domainId);
    if (data.events.filter((event) => event.method === 'NewProof').length > 0) {
        console.log(`Accept proof from invalid user [OnlyOwner]`);
        return ReturnCode.ErrProofFromInvalidSubmitter;
    }

    // Bob can post proof
    data = await submitProof(verifier.pallet, bob, ...verifier.args, domainId);
    if (data.events.filter((event) => event.method === 'NewProof').length === 0) {
        console.log(`Don't proof from valid user [OnlyOwner]`);
        return ReturnCode.ErrProofFromValidSubmitter;
    }

    // OnlyAllowlisted can submit
    data = await registerDomain(bob, 4, 8, "Untrusted", "OnlyAllowlisted", destination, null);
    if (!receivedEvents(data)) {
        console.log(`Register Domain Error`);
        return ReturnCode.ErrDomainRegistrationFailed;
    }

    console.log(`Domain registered successfully: ${data.events}`);
    domainId = data.events[0].data[0];

    let err1 = await submitShouldFail(verifiers[0], alice, domainId, ReturnCode.ErrProofFromInvalidSubmitter, "Accept proof from invalid user (alice) [OnlyAllowlisted]");
    let err2 = await submitShouldFail(verifiers[0], bob, domainId, ReturnCode.ErrProofFromInvalidSubmitter, "Accept proof from invalid user (bob) [OnlyAllowlisted]");
    if (err1 !== ReturnCode.Ok || err2 !== ReturnCode.Ok) {
        return err1;
    }

    data = await allowlistProofSubmitters(bob, domainId, [alice.address]);
    if (!receivedEvents(data)) {
        console.log(`Allowlist proof submitters error`);
        return ReturnCode.ErrAllowlist;
    }

    err1 = await submitShouldFail(verifiers[0], bob, domainId, ReturnCode.ErrProofFromInvalidSubmitter, "Accept proof from invalid user (bob) [OnlyAllowlisted]");
    err2 = await submitShouldSuccess(verifiers[0], alice, domainId, ReturnCode.ErrProofFromInvalidSubmitter, "Reject proof from valid user (alice) [OnlyAllowlisted]");

    if (err1 !== ReturnCode.Ok || err2 !== ReturnCode.Ok) {
        return err1;
    }

    data = await removeProofSubmitters(bob, domainId, [alice.address]);
    if (!receivedEvents(data)) {
        console.log(`Allowlist proof submitters error (remove)`);
        return ReturnCode.ErrAllowlist;
    }

    // Any return value different from 1 is considered an error
    return ReturnCode.Ok;
}

async function submitShouldFail(verifier, who, domainId, error, msg) {
    let data = await submitProof(verifier.pallet, who, ...verifier.args, domainId);
    if (data.events.filter((event) => event.method === 'NewProof').length > 0) {
        console.log(msg);
        return error;
    }
    return ReturnCode.Ok;
}

async function submitShouldSuccess(verifier, who, domainId, error, msg) {
    let data = await submitProof(verifier.pallet, who, ...verifier.args, domainId);
    if (data.events.filter((event) => event.method === 'NewProof').length === 0) {
        console.log(msg);
        return error;
    }
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

