const {ApiPromise, WsProvider, Keyring} = require('@polkadot/api');
const fs = require('fs');
const {decodeAddress} = require('@polkadot/util-crypto');
const {u8aToHex} = require('@polkadot/util');

const WS_ENDPOINT = 'wss://testnet-rpc.zkverify.io';
const DISCARDED_ACCOUNTS_PUBLIC_KEYS = [
    "0x3031dac6e5d70594a193a32fbfd9424953861ef4f348c6150d2576bab660ce7b",
    "0x6a2a7ecf13da1ee5dce50684fd2e4059ccaca63c9b3354e076fead78b648b069",
    "0x2a48b82b0a451b221f21d310a30229eba4aea938e826a229af08b5cc91577132",
    "0x48f2fc2b628c75d8f297b558725f262f13e072d984c61f6a4a1491db363ad432",
    "0x74789a7fb9b37cc6e74cf9599d829a0d8d99b4d9c7bc33388ffda50715218d57",
    "0x028ecaab2910ec4b076999d8af270c7d46b46d493cce8ecf6bdda9cf8a68b66c",
    "0xf63cf0a167de0d342750a7187a961407ad7d74cb67ce93631e93636646a15c69",
    "0xfa9273ff2780b6b2923670792ad5899e9c72ac0dc59526160974f61231177e05",
    "0x5e9c9e3190de7f188613ac26948cd2708b35866704e6bcd5f4180d0275a6d151",
    "0x6e8298157ef6835a12a28c7b13e82e5bc3d327a88a4cffbc5c810223f3a2a91c",
    "0x6d6f646c7a6b2f74727372790000000000000000000000000000000000000000", // Treasury
]

function ss58ToPublicKey(ss58Address) {
    const publicKeyU8a = decodeAddress(ss58Address);
    return u8aToHex(publicKeyU8a);
}

async function takeSnapshot(api, snapshotBalancesFile) {
    const balances = {};
    const accounts = await api.query.system.account.entries();

    for (const [account, data] of accounts) {
        const account_public_key = ss58ToPublicKey(account.slice(-32));
        if (DISCARDED_ACCOUNTS_PUBLIC_KEYS.includes(account_public_key)) continue;
        const {data: {free: freeBalance, reserved: reservedBalance}} = data;
        const totalBalance = BigInt(freeBalance.toString()) + BigInt(reservedBalance.toString());
        balances[account_public_key] = totalBalance.toString();
    }

    const sortedBalances = Object.entries(balances).sort((a, b) => (BigInt(b[1]) < BigInt(a[1]) ? 1 : -1));
    const sortedBalancesObj = sortedBalances.reduce((acc, [account_public_key, balance]) => {
        acc[account_public_key] = balance;
        return acc;
    }, {});

    fs.writeFileSync(snapshotBalancesFile, JSON.stringify(sortedBalancesObj, null, 2));

    console.log('Snapshot saved.');
}

// Restore balances from the snapshot
async function restoreBalances(api, keyring, snapshotBalancesFile, custodySeed) {
    const balances = JSON.parse(fs.readFileSync(snapshotBalancesFile));
    const custodyAccount = keyring.addFromUri(custodySeed);
    let nonce = (await api.query.system.account(custodyAccount.address)).nonce.toNumber();

    const totalAmount = Object.values(balances).reduce((acc, balance) => acc + BigInt(balance), BigInt(0));
    const {data: {free: custodyFreeBalance}} = await api.query.system.account(custodyAccount.address);

    if (BigInt(custodyFreeBalance.toString()) < totalAmount) {
        console.error(`Insufficient balance in custody account. Required: ${totalAmount.toString()}, Available: ${custodyFreeBalance.toString()}`);
        process.exit(1);
    }

    for (const [account, balance] of Object.entries(balances)) {
        const transfer = api.tx.balances.transferAllowDeath(account, balance);
        try {
            const hash = await transfer.signAndSend(custodyAccount, {nonce});
            console.log(`Transferred to ${account}, transaction hash: ${hash.toHex()}`);
            nonce++;
        } catch (error) {
            console.error(`Failed to transfer to ${account}:`, error);
        }
    }
}

async function main() {
    const provider = new WsProvider(WS_ENDPOINT);
    const api = await ApiPromise.create({provider});
    const keyring = new Keyring({type: 'sr25519'});

    const [, , command, arg1, arg2] = process.argv;

    if (command === 'snapshot') {
        if (!arg1) {
            console.error('Please provide the snapshot file name.');
            process.exit(1);
        }
        await takeSnapshot(api, arg1);
    } else if (command === 'restore') {
        if (!arg1 || !arg2) {
            console.error('Please provide the custody seed and snapshot balances file name.');
            process.exit(1);
        }
        await restoreBalances(api, keyring, arg2, arg1);
    } else {
        console.error('Unknown command. Use "snapshot" i.e. "node snapshot-balances.js snapshot snapshot_balances.json" or "restore" i.e. "node snapshot-balances.js restore "//Alice" snapshot_balances.json".');
        process.exit(1);
    }

    process.exit(0);
}

main().catch(console.error);

