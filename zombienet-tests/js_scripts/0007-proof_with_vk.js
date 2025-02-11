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
const { PROOF: ULTRAPLONK_PROOF, PUBS: ULTRAPLONK_PUBS, VK: VK_ULTRAPLONK, VKEY_HASH: ULTRAPLONK_VKEY_HASH,
    STATEMENT_HASH: ULTRAPLONK_STATEMENT_HASH } = require('./ultraplonk_data.js');

async function run(nodeName, networkInfo, _args) {
    const api = await init_api(zombie, nodeName, networkInfo);

    // Create a keyring instance
    const keyring = new zombie.Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');

    // Should accept proof with valid VK
    let events = (await submitProof(api.tx.settlementUltraplonkPallet, alice, { 'Vk': VK_ULTRAPLONK }, ULTRAPLONK_PROOF, ULTRAPLONK_PUBS)).events;
    if (!receivedEvents(events)) {
        return ReturnCode.ErrProofVerificationFailed;
    };
    if (ULTRAPLONK_STATEMENT_HASH != events[0].data[0]) {
        console.log(`Wrong statement hash ${ULTRAPLONK_STATEMENT_HASH} != ${events[0].data[0]}`);
        return ReturnCode.ErrWrongStatementHash;
    }

    // Should reject proof with un unregistered VK hash
    if (receivedEvents(await submitProof(api.tx.settlementUltraplonkPallet, alice, { 'Hash': ULTRAPLONK_VKEY_HASH }, ULTRAPLONK_PROOF, ULTRAPLONK_PUBS))) {
        return ReturnCode.ErrAcceptAnUnregisteredHash;
    };

    events = (await registerVk(api.tx.settlementUltraplonkPallet, alice, VK_ULTRAPLONK)).events;
    if (!receivedEvents(events)) {
        return ReturnCode.ErrVkRegistrationFailed;
    };
    const vkHash = events[0].data[0];
    if (ULTRAPLONK_VKEY_HASH != vkHash) {
        return ReturnCode.ErrWrongKeyHash;
    }

    events = (await submitProof(api.tx.settlementUltraplonkPallet, alice, { 'Hash': ULTRAPLONK_VKEY_HASH }, ULTRAPLONK_PROOF, ULTRAPLONK_PUBS)).events;
    if (!receivedEvents(events)) {
        return ReturnCode.ErrProofVerificationHashFailed;
    };
    if (ULTRAPLONK_STATEMENT_HASH != events[0].data[0]) {
        return ReturnCode.ErrWrongStatementHash;
    }

    // Any return value different from 1 is considered an error
    return ReturnCode.Ok;
}

module.exports = { run }

