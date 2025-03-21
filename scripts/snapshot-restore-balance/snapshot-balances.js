const {ApiPromise, WsProvider, Keyring} = require('@polkadot/api');
const fs = require('fs');
const {decodeAddress} = require('@polkadot/util-crypto');
const {u8aToHex} = require('@polkadot/util');

const DEFAULT_WS_ENDPOINT = 'wss://testnet-rpc.zkverify.io';
const DEFAULT_WS_ENDPOINTS = {
    'local': 'ws://localhost:9944',
    'testnet': DEFAULT_WS_ENDPOINT,
}
const DEFAULT_SNAPSHOT_PATH = 'snapshot_balances.json';
const DEFAULT_FILTERING_PATH = 'filter_account.json';
const DEFAULT_SUDO = ""; // The optional sudo account
const DEFAULT_CAP = 1000000000000000000000; // 1000
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
async function restoreBalances(api, keyring, snapshotBalancesFile, custodyAddress, filterFile, capValue, executor) {
    let balances = JSON.parse(fs.readFileSync(snapshotBalancesFile));
    const filter = JSON.parse(fs.readFileSync(filterFile));
    capValue = BigInt(capValue)
    let nonce = (await api.query.system.account(executor.address)).nonce.toNumber();

    const newBalances = {};

    for (let [account, balance] of Object.entries(balances)) {
        if (filterBalance(account, balance, filter)) continue;
        balance = BigInt(balance);
        if (capValue > 0) {
            if (balance > capValue) {
                console.log(`===> Capping '${account}' for exceeding cap: set it to ${capValue}`);
                balance = capValue;
            }
        }
        newBalances[account] = balance;
    }

    balances = newBalances;

    const totalAmount = Object.values(balances).reduce((acc, balance) => acc + balance, BigInt(0));
    console.log(`Total balance: ${totalAmount.toString()}`);
    const {data: {free: custodyFreeBalance}} = await api.query.system.account(custodyAddress);

    if (BigInt(custodyFreeBalance.toString()) < totalAmount) {
        console.error(`Insufficient balance in custody account. Required: ${totalAmount.toString()}, Available: ${custodyFreeBalance.toString()}`);
        process.exit(1);
    }

    for (const [account, balance] of Object.entries(balances)) {
        console.log(`Transfer '${balance}' to ${account} [${nonce}] `)
        let transfer = api.tx.balances.transferAllowDeath(account, balance);
        if (custodyAddress !== executor.address) {
            // Run as sudo
            transfer = api.tx.sudo.sudoAs(custodyAddress, transfer);
        }
        try {
            const hash = await transfer.signAndSend(executor, {nonce});
            console.log(`Transferred to ${account}, transaction hash: ${hash.toHex()}`);
            nonce++;
        } catch (error) {
            console.error(`Failed to transfer to ${account}:`, error);
        }
    }
}

function usage() {
    console.error(`Usage : snapshot or restore
    node snapshot-balances.js [OPTIONS] snapshot
        or 
    node snapshot-balances.js restore "//CustodyWalletSeed"
        or 
    node snapshot-balances.js -S "//SudoSecret" restore "5H4Rcaj63MBL3fuYNoMuM6XFY2EvAPKD4PasFyCCZA4WrAH3"
    
    OPTIONS:
    -e, --end-point [${DEFAULT_WS_ENDPOINT}] (address or key from ${Object.keys(DEFAULT_WS_ENDPOINTS)})
    -s, --snapshot [${DEFAULT_SNAPSHOT_PATH}]
    -f, --filter [${DEFAULT_FILTERING_PATH}]
    -S, --sudo [${DEFAULT_SUDO}]
    -c, --cap [${DEFAULT_CAP}] = 0 => No cap
    -h, --help : Show this help and exists 
    `);
    process.exit(1);
}

async function main() {
    let args = process.argv.slice(2)
    let wsEndpoint = DEFAULT_WS_ENDPOINT;
    let filter = DEFAULT_FILTERING_PATH;
    let snapshotFile = DEFAULT_SNAPSHOT_PATH;
    let capValue = DEFAULT_CAP;
    let sudo = DEFAULT_SUDO;
    // Skipping command and remove general options like -e
    const newArgs = []
    for (let i = 0; i < args.length - 1; i += 1) {
        if (args[i] === '-e' || args[i] === '--end-point') {
            wsEndpoint = args[++i];
            let address = DEFAULT_WS_ENDPOINTS[wsEndpoint];
            if (address !== undefined) {
                wsEndpoint = address
            }
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
        if (args[i] === '-c' || args[i] === '--cap') {
            capValue = args[++i];
            continue
        }
        if (args[i] === '-S' || args[i] === '--sudo') {
            sudo = args[++i];
            continue
        }
        if (args[i] === '-h' || args[i] === '--help') {
            usage();
        }
        newArgs.push(args[i]);
    }
    if (args.length > 0) {
        newArgs.push(args[args.length - 1]);
    }
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
    if (capValue > 0) {
        console.log(`Using cap: ${capValue}`);
    } else {
        console.log(`No cap`);
    }
    if (sudo !== "") {
        console.log(`Using sudo: ${sudo}`);
    } else {
        console.log(`No sudo`);
    }
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
        let custodyStr = args[0];
        let custodyAddress = null;
        let executor = null;
        if (sudo !== "") {
            executor = keyring.addFromUri(sudo);
            custodyAddress = custodyStr;
        } else {
            executor = keyring.addFromUri(custodyStr);
            custodyAddress = executor.address;
        }
        await restoreBalances(api, keyring, snapshotFile, custodyAddress, filter, capValue, executor);
    } else {
        usage();
    }

    process.exit(0);
}

main().catch(console.error);

