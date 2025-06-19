const {ApiPromise, WsProvider, Keyring} = require('@polkadot/api');
const fs = require('fs');
const {decodeAddress} = require('@polkadot/util-crypto');
const {u8aToHex} = require('@polkadot/util');

const DEFAULT_PALLETS = ['groth16', 'ultraplonk', 'risc0'];
const DEFAULT_WS_ENDPOINT = 'wss://testnet-rpc.zkverify.io';
const DEFAULT_WS_ENDPOINTS = {
    'local': 'ws://localhost:9944',
    'testnet': DEFAULT_WS_ENDPOINT,
}
const DEFAULT_SNAPSHOT_PATH = 'snapshot_vks.json';

function pallet(sub, p) {
    return {
        'groth16': sub.settlementGroth16Pallet,
        'ultraplonk': sub.settlementUltraplonkPallet,
        'risc0': sub.settlementRisc0Pallet
    }[p]
}

async function takeSnapshot(api, pallets, snapshotVksFile) {
    const vks = {};
    console.log(`pallets '${pallets}'.`);
    for (const p of pallets) {
        console.log(`Get vks in pallet ${p}.`);
        const vkeys = await pallet(api.query, p).vks.entries()
        const tickets = await pallet(api.query, p).tickets.entries()
        vks[p] = {};
        vkeys.forEach(([{args: [hash]}, data]) => {
            const {vk: vk} = JSON.parse(data);
            vks[p][hash] = {
                vk: vk,
                owners: []
            };
        });

        tickets.forEach(([{args: [[owner, hash]]}, _value]) => {
            vks[p][hash].owners.push(owner);
        });

    }
    fs.writeFileSync(snapshotVksFile, JSON.stringify(vks, null, 2));
    console.log('Snapshot saved.');
}

async function restore(api, keyring, pallets, snapshotVksFile, sudo) {
    let vksMap = JSON.parse(fs.readFileSync(snapshotVksFile));
    let nonce = (await api.query.system.account(sudo.address)).nonce.toNumber();

    for (let [p, vks] of Object.entries(vksMap)) {
        for (const [_hash, {vk, owners}] of Object.entries(vks)) {
            for (const owner of owners) {
                console.log(`Register vk on '${p} for owner '${owner}' [${nonce}]`)
                // Run as sudo
                let registerVk = pallet(api.tx, p).registerVk(vk);
                registerVk = api.tx.sudo.sudoAs(owner, registerVk);

                try {
                    const hash = await registerVk.signAndSend(sudo, {nonce});
                    console.log(`Registered vk on pallet ${p}, transaction hash: ${hash.toHex()}`);
                    nonce++;
                } catch (error) {
                    console.error(`Failed to register vk on ${p}:`, error);
                }

            }
        }
    }
}

function usage() {
    console.error(`Usage : snapshot or restore vk
    node snapshot-vks.js [OPTIONS] snapshot
        or 
    node snapshot-vks.js restore "//SudoSeed"
    
    OPTIONS:
    -e, --end-point [${DEFAULT_WS_ENDPOINT}] (address or key from ${Object.keys(DEFAULT_WS_ENDPOINTS)})
    -s, --snapshot [${DEFAULT_SNAPSHOT_PATH}]
    -p, --pallets [${DEFAULT_PALLETS}]
    -h, --help : Show this help and exists 
    `);
    process.exit(1);
}

async function main() {
    let args = process.argv.slice(2)
    let wsEndpoint = DEFAULT_WS_ENDPOINT;
    let snapshotFile = DEFAULT_SNAPSHOT_PATH;
    let pallets = DEFAULT_PALLETS;
    // Skipping command and remove general options like -e
    const newArgs = [];
    let i = 0;
    for (; i < args.length - 1; i += 1) {
        if (args[i] === '-e' || args[i] === '--end-point') {
            wsEndpoint = args[++i];
            let address = DEFAULT_WS_ENDPOINTS[wsEndpoint];
            if (address !== undefined) {
                wsEndpoint = address
            }
            continue
        }
        if (args[i] === '-s' || args[i] === '--snapshot') {
            snapshotFile = args[++i];
            continue
        }
        if (args[i] === '-p' || args[i] === '--pallets') {
            pallets = args[++i].split(',').map(s => s.trim());
            continue
        }
        if (args[i] === '-h' || args[i] === '--help') {
            usage();
        }
        newArgs.push(args[i]);
    }
    if (args.length - i > 0) {
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
    console.log(`Using pallets: ${pallets}`);
    const provider = new WsProvider(wsEndpoint);
    const api = await ApiPromise.create({provider});
    const keyring = new Keyring({type: 'sr25519'});

    if (command === 'snapshot') {
        console.log(`Dump vks on ${snapshotFile}`);
        await takeSnapshot(api, pallets, snapshotFile);
    } else if (command === 'restore') {
        if (args.length !== 1) {
            usage();
        }
        const sudo = keyring.addFromUri(args[0]);
        await restore(api, keyring, pallets, snapshotFile, sudo);
    } else {
        usage();
    }

    process.exit(0);
}

main().catch(console.error);

