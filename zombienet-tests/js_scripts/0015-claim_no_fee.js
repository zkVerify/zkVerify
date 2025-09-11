const { init_api, sudoInitClaim, claim, receivedEvents, submitExtrinsic, BlockUntil } = require('zkv-lib');
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
    console.log('beneficiary address:', beneficiary_address);

    // Build Beneficiaries Map
    const beneficiaries_map = new Map();
    beneficiaries_map.set(beneficiary_address, '1000000000000000000');

    // Begin claim with beneficiary
    let events = await sudoInitClaim(alice, beneficiaries_map, '10000000000000000000');
    if (!receivedEvents(events)) {
        console.log(`Failed to initialize claim`);
        return ReturnCode.ErrInitClaim;
    }

    const keys = await api.query.claim.beneficiaries.keys();
    const beneficiaries = keys.map(({ args: [beneficiaryId] }) => beneficiaryId);
    console.log('all beneficiaries:', beneficiaries.join(', '));

    // Attempt to claim
    events = await claim(beneficiary);
    if (!receivedEvents(events)) {
        console.log(`Failed to claim`);
        return ReturnCode.ErrClaim;
    }

    // Check beneficiary balance
    let balance_beneficiary = (await api.query.system.account(beneficiary_address))["data"]["free"];
    if (balance_beneficiary != '1000000000000000000') {
        console.log(`Beneficiary balance is ${balance_beneficiary}, expected 1000000000000000000`);
        return ReturnCode.ErrClaimedAmount;
    }

    return ReturnCode.Ok;
}

module.exports = { run }