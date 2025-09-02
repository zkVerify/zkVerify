const { init_api, sudoInitClaim, claim, receivedEvents } = require('zkv-lib');
const ReturnCode = {
    Ok: 1,
    ErrInitClaim: 2,
    ErrClaim: 3,
    ErrClaimedAmount: 4,
};

async function run(nodeName, networkInfo, _args) {

    const api = await init_api(zombie, nodeName, networkInfo);

    // Build a keyring and import Alice's credential
    const keyring = new zombie.Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');

    // Build a beneficiary starting from 0 balance
    const beneficiary = keyring.addFromUri('//Beneficiary');
    const beneficiary_address = beneficiary.address;
    const beneficiaries_map = new Map();
    beneficiaries_map.set(beneficiary_address, '1');

    // Begin airdrop with beneficiary
    let events = await sudoInitClaim(alice, beneficiaries_map, '10');
    if (!receivedEvents(events)) {
        console.log(`Failed to initialize claim`);
        return ReturnCode.ErrInitClaim;
    }

    // Attempt to claim
    events = await claim(beneficiary);
    if (!receivedEvents(events)) {
        console.log(`Failed to claim`);
        return ReturnCode.ErrClaim;
    }

    // Check beneficiary balance
    let balance_beneficiary = (await api.query.system.account(beneficiary_address))["data"]["free"];
    if (balance_beneficiary != '1') {
        console.log(`Beneficiary balance is ${balance_beneficiary}, expected 1`);
        return ReturnCode.ErrClaimedAmount;
    }

    return ReturnCode.Ok;
}

module.exports = { run }