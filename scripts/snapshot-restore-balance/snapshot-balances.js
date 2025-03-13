const {ApiPromise, WsProvider, Keyring} = require('@polkadot/api');
const fs = require('fs');
const {decodeAddress} = require('@polkadot/util-crypto');
const {u8aToHex} = require('@polkadot/util');

const DEFAULT_WS_ENDPOINT = 'wss://testnet-rpc.zkverify.io';
const DEFAULT_SNAPSHOT_PATH = 'snapshot_balances.json';
const DEFAULT_FILTERING_PATH = 'filter_account.json';
// const WS_ENDPOINT = 'ws://localhost:9944';
const EXISTENTIAL_DEPOSIT = 10000000000000000; //0.01
// Don't recover all accounts that ends with "0000000000000000000000000000000000000000"
const MODULE_ACCOUNT_ENDS = "0000000000000000000000000000000000000000";
const FILTER_MODULE_ACCOUNT = true;

function ss58ToPublicKey(ss58Address) {
    const publicKeyU8a = decodeAddress(ss58Address);
    return u8aToHex(publicKeyU8a);
}

async function takeSnapshot(api, snapshotBalancesFile) {
    const balances = {};
    const accounts = await api.query.system.account.entries();

    for (const [account, data] of accounts) {
        const account_public_key = ss58ToPublicKey(account.slice(-32));
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

function filterBalance(account, balance, blockList) {
    if (balance < EXISTENTIAL_DEPOSIT) {
        console.log(`===> Skip '${account}' for lower balance ${balance} < ${EXISTENTIAL_DEPOSIT}`);
        return true;
    }
    if (blockList.includes(account)) {
        console.log(`===> Filter blockList '${account}'`)
        return true;
    }
    if (FILTER_MODULE_ACCOUNT && account.endsWith(MODULE_ACCOUNT_ENDS)) {
        console.log(`===> Skip module account '${account}'`)
        return true;
    }
    return false;
}

// Restore balances from the snapshot
async function restoreBalances(api, keyring, snapshotBalancesFile, custodySeed, filterFile) {
    let balances = JSON.parse(fs.readFileSync(snapshotBalancesFile));
    const filter = JSON.parse(fs.readFileSync(filterFile));
    const custodyAccount = keyring.addFromUri(custodySeed);
    let nonce = (await api.query.system.account(custodyAccount.address)).nonce.toNumber();

    const newBalances = {};

    for (const [account, balance] of Object.entries(balances)) {
        if (filterBalance(account, balance, filter)) continue;
        newBalances[account] = balance;
    }

    balances = newBalances;

    const totalAmount = Object.values(balances).reduce((acc, balance) => acc + BigInt(balance), BigInt(0));
    console.log(`Total balance: ${totalAmount.toString()}`);
    const {data: {free: custodyFreeBalance}} = await api.query.system.account(custodyAccount.address);

    if (BigInt(custodyFreeBalance.toString()) < totalAmount) {
        console.error(`Insufficient balance in custody account. Required: ${totalAmount.toString()}, Available: ${custodyFreeBalance.toString()}`);
        process.exit(1);
    }

    for (const [account, balance] of Object.entries(balances)) {
        console.log(`Transfer '${balance}' to ${account} [${nonce}] `)
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

function usage() {
    console.error(`Usage : snapshot or restore
    node snapshot-balances.js [-e ] snapshot
        or 
    node snapshot-balances.js restore "//Alice"
    
    -e, --end-point [${DEFAULT_WS_ENDPOINT}]
    -s, --snapshot [${DEFAULT_SNAPSHOT_PATH}]
    -f, --filter [${DEFAULT_FILTERING_PATH}]
    `);
    process.exit(1);
}

async function main() {
    let args = process.argv.slice(2)
    let wsEndpoint = DEFAULT_WS_ENDPOINT;
    let filter = DEFAULT_FILTERING_PATH;
    let snapshotFile = DEFAULT_SNAPSHOT_PATH;
    // Skipping command and remove general options like -e
    const newArgs = []
    for (let i = 0; i < args.length - 1; i += 1) {
        if (args[i] === '-e' || args[i] === '--end-point') {
            wsEndpoint = args[++i];
            continue
        }
        if (args[i] === '-f' || args[i] === '--filter') {
            filter = args[++i];
            continue
        }
        if (args[i] === '-s' || args[i] === '--snapshot') {
            snapshotFile = args[++i];
            continue
        }
        if (args[i] === '-h' || args[i] === '--help') {
            usage();
        }
        newArgs.push(args[i]);
    }
    newArgs.push(args[args.length - 1]);
    args = newArgs;
    if (args.length < 1) {
        usage();
    }
    const command = args[0].toLowerCase();
    args = args.slice(1);
    console.log(`command: ${command}`);
    console.log(`Using WS endpoint: ${wsEndpoint}`);
    console.log(`Using snapshot: ${snapshotFile}`);
    console.log(`Using filter: ${filter}`);
    const provider = new WsProvider(wsEndpoint);
    const api = await ApiPromise.create({provider});
    const keyring = new Keyring({type: 'sr25519'});

    if (command === 'snapshot') {
        console.log(`Dump balances on ${snapshotFile}`);
        await takeSnapshot(api, snapshotFile);
    } else if (command === 'restore') {
        if (args.length !== 1) {
            usage();
        }
        const custodySeed = args[0];
        await restoreBalances(api, keyring, snapshotFile, custodySeed, filter);
    } else {
        usage();
    }

    process.exit(0);
}

main().catch(console.error);

