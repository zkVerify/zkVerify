const ReturnCode = {
    Ok: 1,
    ErrProofVerificationFailed: 2,
    ErrAcceptAnUnregisteredHash: 3,
    ErrVkRegistrationFailed: 4,
    ErrWrongKeyHash: 5,
    ErrProofVerificationHashFailed: 6,
    ErrWrongStatementHash: 7,
};

const { init_api, submitProof, registerVk, receivedEvents } = require('zkv-lib')
const { PROOF: STWO_PROOF, PUBS: STWO_PUBS, VK: VK_STWO, VKEY_HASH: STWO_VKEY_HASH,
    STATEMENT_HASH: STWO_STATEMENT_HASH } = require('./stwo_data.js');

async function run(nodeName, networkInfo, _args) {
    const api = await init_api(zombie, nodeName, networkInfo);

    // Create a keyring instance
    const keyring = new zombie.Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');

    console.log('Testing STARK (Stwo) verifier e2e...');

    // Should accept proof with valid VK
    console.log('Testing proof submission with VK...');
    let events = (await submitProof(api.tx.settlementStwoPallet, alice, { 'Vk': VK_STWO }, STWO_PROOF, STWO_PUBS)).events;
    if (!receivedEvents(events)) {
        console.log('Proof verification failed with VK');
        return ReturnCode.ErrProofVerificationFailed;
    };
    if (STWO_STATEMENT_HASH != events[0].data[0]) {
        console.log(`Wrong statement hash ${STWO_STATEMENT_HASH} != ${events[0].data[0]}`);
        return ReturnCode.ErrWrongStatementHash;
    }
    console.log('✓ Proof verification with VK successful');

    // Should reject proof with unregistered VK hash
    console.log('Testing proof submission with unregistered hash...');
    if (receivedEvents(await submitProof(api.tx.settlementStwoPallet, alice, { 'Hash': STWO_VKEY_HASH }, STWO_PROOF, STWO_PUBS))) {
        console.log('Proof verification should have failed with unregistered hash');
        return ReturnCode.ErrAcceptAnUnregisteredHash;
    };
    console.log('✓ Proof correctly rejected with unregistered hash');

    console.log('Registering STARK VK...');

    events = (await registerVk(api.tx.settlementStwoPallet, alice, VK_STWO)).events;
    if (!receivedEvents(events)) {
        console.log('Failed to register STARK VK');
        return ReturnCode.ErrVkRegistrationFailed;
    };

    const vkHash = events[0].data[0];
    if (STWO_VKEY_HASH != vkHash) {
        console.log(`Wrong VK hash ${STWO_VKEY_HASH} != ${vkHash}`);
        return ReturnCode.ErrWrongKeyHash;
    }
    console.log('✓ STARK VK registration successful');

    // Should accept proof with registered VK hash
    console.log('Testing proof submission with registered hash...');
    events = (await submitProof(api.tx.settlementStwoPallet, alice, { 'Hash': STWO_VKEY_HASH }, STWO_PROOF, STWO_PUBS)).events;
    if (!receivedEvents(events)) {
        console.log('Proof verification failed with registered hash');
        return ReturnCode.ErrProofVerificationHashFailed;
    };
    if (STWO_STATEMENT_HASH != events[0].data[0]) {
        console.log(`Wrong statement hash ${STWO_STATEMENT_HASH} != ${events[0].data[0]}`);
        return ReturnCode.ErrWrongStatementHash;
    }
    console.log('✓ Proof verification with registered hash successful');

    console.log('STARK verifier e2e test completed successfully!');
    return ReturnCode.Ok;
}

module.exports = { run }
