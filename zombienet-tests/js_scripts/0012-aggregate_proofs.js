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
    ErrProofOnUnregisteredDomain: 8,
    ErrWrongDomainId: 9,
    ErrPublishShouldNotPay: 10,
};

const { init_api, submitProof, receivedEvents, registerDomain, unregisterDomain, aggregate, getBalance } = require('zkv-lib');
const { PROOF: ZKSYNC_PROOF, PUBS: ZKSYNC_PUBS } = require('./zksync_data.js');
const { PROOF: FFLONK_PROOF, PUBS: FFLONK_PUBS, VK: FFLONK_VK } = require('./fflonk_data.js');
const { PROOF: GROTH16_PROOF, PUBS: GROTH16_PUBS, VK: GROTH16_VK } = require('./groth16_data.js');
const { PROOF: RISC0_PROOF, PUBS: RISC0_PUBS, VK: RISC0_VK } = require('./risc0_data.js');
const { PROOF: ULTRAPLONK_PROOF, PUBS: ULTRAPLONK_PUBS, VK: ULTRAPLONK_VK } = require('./ultraplonk_data.js');
const { PROOF: PROOFOFSQL_PROOF, PUBS: PROOFOFSQL_PUBS, VK: PROOFOFSQL_VK } = require('./proofofsql_data.js');

async function run(nodeName, networkInfo, _args) {
    const api = await init_api(zombie, nodeName, networkInfo);

    // Create a keyring instance
    const keyring = new zombie.Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');
    const bob = keyring.addFromUri('//Bob');

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
            name: "Risc0",
            pallet: api.tx.settlementRisc0Pallet,
            args: [{ 'Vk': RISC0_VK }, RISC0_PROOF, RISC0_PUBS],
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
        }
    ];

    let events = await registerDomain(bob, verifiers.length, null);
    if (!receivedEvents(events)) {
        console.log(`Register Domain Error`);
        return ReturnCode.ErrDomainRegistrationFailed;
    }
    let domainId = events.events[0].data[0];
    console.log(`Domain registered successfully: ${events.events}`);

    let failed = false;
    let readyToAggregate = [];
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
        readyToAggregate = readyToAggregate.concat(verifier.data.events.filter((event) => event.section === 'aggregate' && event.method === 'ReadyToAggregate'));
    }

    if (failed) {
        return ReturnCode.ErrProofVerificationFailed;
    }

    if (readyToAggregate.length === 0) {
        console.log(`No aggregation to aggregate`);
        return ReturnCode.ErrAggregationNotReady;
    }

    let receipts = [];

    for (const event of readyToAggregate) {
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

    events = await unregisterDomain(bob, domainId);
    if (!receivedEvents(events)) {
        console.log(`Unregister Domain Error: bob should unregister the domain`);
        return ReturnCode.ErrDomainUnregistrationFailed;
    }

    data = await submitProof(verifiers[0].pallet, alice, ...verifiers[0].args, domainId);
    if (data.events.filter((event) => event.method === 'NewProof').length > 0) {
        console.log(`Accept proof on unregistered domain`);
        return ReturnCode.ErrProofOnUnregisteredDomain;
    }

    data = await registerDomain(bob, 16, 8);
    if (!receivedEvents(data)) {
        console.log(`Register Domain Error`);
        return ReturnCode.ErrDomainRegistrationFailed;
    }
    console.log(`Domain registered successfully: ${events.events}`);
    let newDomainId = data.events[0].data[0];
    if (newDomainId <= domainId) {
        console.log(`Domain registered with wrong id`);
        return ReturnCode.ErrWrongDomainId;
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

