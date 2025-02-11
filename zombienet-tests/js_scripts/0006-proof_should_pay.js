const MIN_PRICE = 1000000000;

const ReturnCode = {
    Ok: 1,
    ErrProofVerificationFailed: 2,
    ErrFalseProofVerified: 3,
    ErrValidProofNotPay: 4,
    ErrInvalidProofNotPay: 5,
};

const { init_api, submitProof, getBalance, receivedEvents } = require('zkv-lib')
const { PROOF, PUBS, VK } = require('./ultraplonk_data.js');

async function run(nodeName, networkInfo, _args) {
    const api = await init_api(zombie, nodeName, networkInfo);

    // Create a keyring instance
    const keyring = new zombie.Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');

    let balanceAlice = await getBalance(alice);
    console.log('Alice\'s balance: ' + balanceAlice.toHuman());

    if (!receivedEvents(await submitProof(api.tx.settlementUltraplonkPallet, alice, { 'Vk': VK }, PROOF, PUBS))) {
        return ReturnCode.ErrProofVerificationFailed;
    };

    let newBalanceAlice = await getBalance(alice);
    console.log('Alice\'s balance after valid proof: ' + newBalanceAlice.toHuman());

    if (balanceAlice - newBalanceAlice <= MIN_PRICE) {
        return ReturnCode.ErrValidProofNotPay;
    }

    balanceAlice = newBalanceAlice;

    if (receivedEvents(await submitProof(api.tx.settlementUltraplonkPallet, alice, { 'Vk': VK }, PROOF, 0))) {
        return ReturnCode.ErrFalseProofVerified;
    };

    newBalanceAlice = await getBalance(alice);
    console.log('Alice\'s balance after invalid proof: ' + newBalanceAlice.toHuman());

    if (balanceAlice - newBalanceAlice <= MIN_PRICE) {
        return ReturnCode.ErrInvalidProofNotPay;
    }

    // Any return value different from 1 is considered an error
    return ReturnCode.Ok;
}

module.exports = { run }

