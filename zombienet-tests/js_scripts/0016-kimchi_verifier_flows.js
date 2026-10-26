const MIN_PRICE = 1000000000;
const KIMCHI_SECTION = "settlementKimchiPallet";

const ReturnCode = {
    Ok: 1,
    ErrInlineProofVerificationFailed: 2,
    ErrAcceptedUnregisteredHash: 3,
    ErrVkRegistrationFailed: 4,
    ErrMissingVkHash: 5,
    ErrProofVerificationHashFailed: 6,
    ErrVkUnregistrationFailed: 7,
    ErrAcceptedHashAfterUnregister: 8,
    ErrShapeProofVerificationFailed: 9,
    ErrFalseProofVerified: 10,
    ErrValidProofNotPay: 11,
    ErrInvalidProofNotPay: 12,
};

const { init_api, submitProof, submitExtrinsic, BlockUntil, getBalance, receivedEvents } = require('zkv-lib');
const {
    ONE_CHUNK_SIMPLE_PUBS_0,
    TWO_CHUNK_LOOKUP_RUNTIME_PUBS_64,
    FOUR_CHUNK_ALL_FEATURES_PUBS_0,
    FOUR_CHUNK_ALL_FEATURES_PUBS_1024,
} = require('./kimchi_data.js');

function proofArgsFromVk(fixture) {
    return [{ 'Vk': fixture.VK }, fixture.PROOF, fixture.PUBS];
}

function proofArgsFromHash(vkHash, fixture) {
    return [{ 'Hash': vkHash }, fixture.PROOF, fixture.PUBS];
}

function corruptPubs(pubs) {
    const corrupted = pubs.slice();
    const field = Buffer.from(corrupted[0].slice(2), 'hex');
    field[0] ^= 1;
    corrupted[0] = '0x' + field.toString('hex');
    return corrupted;
}

async function registerKimchiVk(api, signer, vk) {
    return await submitExtrinsic(api, api.tx.settlementKimchiPallet.registerVk(vk), signer, BlockUntil.InBlock,
        (event) => event.section == KIMCHI_SECTION && event.method == "VkRegistered"
    );
}

async function unregisterKimchiVk(api, signer, vkHash) {
    return await submitExtrinsic(api, api.tx.settlementKimchiPallet.unregisterVk(vkHash), signer, BlockUntil.InBlock,
        (event) => event.section == KIMCHI_SECTION && event.method == "VkUnregistered"
    );
}

async function submitValidKimchiProof(pallet, signer, name, fixture) {
    console.log(`Submitting Kimchi proof: ${name}`);
    const result = await submitProof(pallet, signer, ...proofArgsFromVk(fixture));
    if (!receivedEvents(result)) {
        console.log(`Kimchi proof was not verified: ${name}`);
        return false;
    }
    return true;
}

async function run(nodeName, networkInfo, _args) {
    const api = await init_api(zombie, nodeName, networkInfo);

    const keyring = new zombie.Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');
    const kimchi = api.tx.settlementKimchiPallet;

    if (!await submitValidKimchiProof(kimchi, alice, "two-chunk lookup-runtime inline VK", TWO_CHUNK_LOOKUP_RUNTIME_PUBS_64)) {
        return ReturnCode.ErrInlineProofVerificationFailed;
    }

    if (receivedEvents(await submitProof(kimchi, alice, ...proofArgsFromHash('0x' + '00'.repeat(32), TWO_CHUNK_LOOKUP_RUNTIME_PUBS_64)))) {
        return ReturnCode.ErrAcceptedUnregisteredHash;
    }

    console.log(`Registering Kimchi VK`);
    const registerResult = await registerKimchiVk(api, alice, TWO_CHUNK_LOOKUP_RUNTIME_PUBS_64.VK);
    if (!receivedEvents(registerResult)) {
        return ReturnCode.ErrVkRegistrationFailed;
    }

    const vkHash = registerResult.events[0].data[0].toString();
    if (!vkHash) {
        return ReturnCode.ErrMissingVkHash;
    }

    console.log(`Submitting Kimchi proof by registered VK hash: ${vkHash}`);
    if (!receivedEvents(await submitProof(kimchi, alice, ...proofArgsFromHash(vkHash, TWO_CHUNK_LOOKUP_RUNTIME_PUBS_64)))) {
        return ReturnCode.ErrProofVerificationHashFailed;
    }

    console.log(`Unregistering Kimchi VK`);
    if (!receivedEvents(await unregisterKimchiVk(api, alice, vkHash))) {
        return ReturnCode.ErrVkUnregistrationFailed;
    }

    if (receivedEvents(await submitProof(kimchi, alice, ...proofArgsFromHash(vkHash, TWO_CHUNK_LOOKUP_RUNTIME_PUBS_64)))) {
        return ReturnCode.ErrAcceptedHashAfterUnregister;
    }

    const extraShapes = [
        ["one-chunk simple pubs=0", ONE_CHUNK_SIMPLE_PUBS_0],
        ["four-chunk all-features pubs=0", FOUR_CHUNK_ALL_FEATURES_PUBS_0],
        ["four-chunk all-features pubs=1024", FOUR_CHUNK_ALL_FEATURES_PUBS_1024],
    ];
    for (const [name, fixture] of extraShapes) {
        if (!await submitValidKimchiProof(kimchi, alice, name, fixture)) {
            return ReturnCode.ErrShapeProofVerificationFailed;
        }
    }

    let balanceAlice = await getBalance(alice);
    console.log('Alice\'s balance before valid Kimchi fee check: ' + balanceAlice.toHuman());

    if (!receivedEvents(await submitProof(kimchi, alice, ...proofArgsFromVk(TWO_CHUNK_LOOKUP_RUNTIME_PUBS_64)))) {
        return ReturnCode.ErrInlineProofVerificationFailed;
    }

    let newBalanceAlice = await getBalance(alice);
    console.log('Alice\'s balance after valid Kimchi proof: ' + newBalanceAlice.toHuman());

    if (balanceAlice - newBalanceAlice <= MIN_PRICE) {
        return ReturnCode.ErrValidProofNotPay;
    }

    balanceAlice = newBalanceAlice;

    if (receivedEvents(await submitProof(
        kimchi,
        alice,
        { 'Vk': TWO_CHUNK_LOOKUP_RUNTIME_PUBS_64.VK },
        TWO_CHUNK_LOOKUP_RUNTIME_PUBS_64.PROOF,
        corruptPubs(TWO_CHUNK_LOOKUP_RUNTIME_PUBS_64.PUBS)
    ))) {
        return ReturnCode.ErrFalseProofVerified;
    }

    newBalanceAlice = await getBalance(alice);
    console.log('Alice\'s balance after invalid Kimchi proof: ' + newBalanceAlice.toHuman());

    if (balanceAlice - newBalanceAlice <= MIN_PRICE) {
        return ReturnCode.ErrInvalidProofNotPay;
    }

    return ReturnCode.Ok;
}

module.exports = { run }
