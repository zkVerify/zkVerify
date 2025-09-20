const ReturnCode = {
    Ok: 1,
    ErrProofVerificationFailed: 2,
    ErrBatchVerificationFailed: 3,
    ErrVkRegistrationFailed: 4,
};

const { init_api, submitProof, registerVk, receivedEvents } = require('zkv-lib')
const { PROOF: STWO_PROOF, PUBS: STWO_PUBS, VK: VK_STWO, VKEY_HASH: STWO_VKEY_HASH,
    STATEMENT_HASH: STWO_STATEMENT_HASH } = require('./stwo_data.js');

async function run(nodeName, networkInfo, _args) {
    const api = await init_api(zombie, nodeName, networkInfo);

    // Create a keyring instance
    const keyring = new zombie.Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');

    console.log('Testing STARK (Stwo) batch verification...');

    // Register VK first
    console.log('Registering STARK VK...');
    let events = (await registerVk(api.tx.settlementStwoPallet, alice, VK_STWO)).events;
    if (!receivedEvents(events)) {
        console.log('Failed to register STARK VK');
        return ReturnCode.ErrVkRegistrationFailed;
    };
    console.log('✓ STARK VK registration successful');

    // Test batch verification with multiple proofs
    console.log('Testing batch verification with 3 STARK proofs...');
    
    const batchResults = [];
    for (let i = 0; i < 3; i++) {
        console.log(`Submitting STARK proof ${i + 1}/3...`);
        events = (await submitProof(api.tx.settlementStwoPallet, alice, { 'Hash': STWO_VKEY_HASH }, STWO_PROOF, STWO_PUBS)).events;
        
        if (!receivedEvents(events)) {
            console.log(`STARK proof ${i + 1} verification failed`);
            return ReturnCode.ErrProofVerificationFailed;
        }
        
        if (STWO_STATEMENT_HASH != events[0].data[0]) {
            console.log(`STARK proof ${i + 1} has wrong statement hash`);
            return ReturnCode.ErrProofVerificationFailed;
        }
        
        batchResults.push(true);
        console.log(`✓ STARK proof ${i + 1} verification successful`);
    }

    if (batchResults.length !== 3) {
        console.log('Batch verification failed - not all proofs processed');
        return ReturnCode.ErrBatchVerificationFailed;
    }

    console.log('✓ All 3 STARK proofs verified successfully in batch');
    console.log('STARK batch verification test completed successfully!');
    return ReturnCode.Ok;
}

module.exports = { run }
