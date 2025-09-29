const {ApiPromise, WsProvider, Keyring} = require('@polkadot/api');
const fs = require('fs');
const {decodeAddress} = require('@polkadot/util-crypto');
const {u8aToHex} = require('@polkadot/util');
const {gql, request} = require("graphql-request");

const DEFAULT_PALLETS = ['fflonk', 'groth16', 'plonky2', 'risc0', 'sp1', 'ultrahonk', 'ultraplonk'];
const DEFAULT_WS_ENDPOINT = 'wss://zkverify-volta-rpc.zkverify.io';
const DEFAULT_WS_ENDPOINTS = {
    'local': 'ws://localhost:9944',
    'testnet': DEFAULT_WS_ENDPOINT,
}
const DEFAULT_SNAPSHOT_PATH = 'snapshot_vks.json';
const DEFAULT_GRAPHQL = 'https://zkpvnetwork.squids.live/zkverify@v8/api/graphql';

function pallet(sub, p) {
    return {
        'fflonk': sub.settlementFFlonkPallet,
        'groth16': sub.settlementGroth16Pallet,
        'plonky2': sub.settlementPlonky2Pallet,
        'risc0': sub.settlementRisc0Pallet,
        'sp1': sub.settlementSp1Pallet,
        'ultrahonk': sub.settlementUltrahonkPallet,
        'ultraplonk': sub.settlementUltraplonkPallet
    }[p]
}

function palletToGraphQl(p) {
    return {
        'fflonk': 'SettlementFFlonkPallet',
        'groth16': 'SettlementGroth16Pallet',
        'plonky2': 'SettlementPlonky2Pallet',
        'risc0': 'SettlementRisc0Pallet',
        'sp1': 'SettlementSp1Pallet',
        'ultrahonk': 'SettlementUltrahonkPallet',
        'ultraplonk': 'SettlementUltraplonkPallet'
    }[p]
}

async function getVkStats(gqlEndpoint, pallet, vkey) {
    const query = gql`
      query SubmittedProofs($pallet: String, $vkey: String) {
        extrinsics(
          condition: {pallet: $pallet, call: "submit_proof"}
          filter: {vkHash: {equalTo: $vkey}}
          orderBy: BLOCK_NUMBER_DESC
          first: 1
        ) {
          totalCount
          edges {
            node {
              blockNumber
              vkHash
              timestamp
            }
          }
        }
      }
    `;

    const variables = {
        pallet,
        vkey
    };


    let out = await request(gqlEndpoint, query, variables);
    let total = out.extrinsics.totalCount;
    var lastUsed = null;
    if (total > 0) {
        lastUsed = out.extrinsics.edges[0].node.timestamp;
    }

    return {
        total,
        lastUsed,
    }
}

async function takeSnapshot(api, pallets, snapshotVksFile, gqlEndpoint = DEFAULT_GRAPHQL) {
    const vks = {};
    console.log(`pallets '${pallets}'.`);
    for (const p of pallets) {
        console.log(`Get vks in pallet ${p}.`);
        const vkeys = await pallet(api.query, p).vks.entries();
        const tickets = await pallet(api.query, p).tickets.entries();
        vks[p] = {};
        for (const [{args: [hash]}, data] of vkeys) {
            const {vk: vk} = JSON.parse(data);
            let stat = null;
            if (gqlEndpoint !== null) {
                stat = await getVkStats(gqlEndpoint, palletToGraphQl(p), hash);
                console.log(
                    `vk ${hash} in pallet ${p} has ${stat.total} submissions, last used ${stat.lastUsed}.`
                )
            }
            vks[p][hash] = {
                vk: vk,
                owners: [],
                bonded: 0.0,
                stat
            };
        }

        let bonded = 0.0;
        tickets.forEach(([{args: [[owner, hash]]}, value]) => {
            if (vks[p][hash].owners.length === 0) {
                let b = value / 1000000000000000000.0;
                vks[p][hash].bonded = b;
                bonded += b;
            }
            vks[p][hash].owners.push(owner);
        });
        console.log(`Need bond ${bonded} VFY. `);
    }
    fs.writeFileSync(snapshotVksFile, JSON.stringify(vks, null, 2));
    console.log('Snapshot saved.');
}

async function restore(api, keyring, pallets, snapshotVksFile, sudo, replace = null, usedAfter = null) {
    let vksMap = JSON.parse(fs.readFileSync(snapshotVksFile));
    let nonce = (await api.query.system.account(sudo.address)).nonce.toNumber();

    for (let [p, vks] of Object.entries(vksMap)) {
        for (let [hash, {vk, owners, stat}] of Object.entries(vks)) {
            if (usedAfter !== null) {
                if (stat.lastUsed === null || new Date(stat.lastUsed) < usedAfter) {
                    console.log(`Skip vk ${hash} in pallet ${p} because it was not used after ${usedAfter}`);
                    continue;
                }
            }
            if (replace !== undefined && replace !== false && replace !== null) {
                owners = [replace];
            }
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
    -r, --replace = null => Do not replace owner otherwise use this address instead
    -u, --used Recover only the vk that are used at least once
    -U, --used-after Recover only the vk that are used after the given date
    -q, --gql [${DEFAULT_GRAPHQL}] => GraphQL endpoint to get vk usage statistics
    -h, --help : Show this help and exists 
    `);
    process.exit(1);
}

async function main() {
    let args = process.argv.slice(2)
    let wsEndpoint = DEFAULT_WS_ENDPOINT;
    let snapshotFile = DEFAULT_SNAPSHOT_PATH;
    let pallets = DEFAULT_PALLETS;
    let gqlEndpoint = DEFAULT_GRAPHQL;
    let replace = null;
    let usedAfter = null;
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
        if (args[i] === '-q' || args[i] === '--gql') {
            gqlEndpoint = args[++i];
            if (gqlEndpoint === 'null') {
                gqlEndpoint = null;
            }
            continue
        }
        if (args[i] === '-r' || args[i] === '--replace') {
            replace = args[++i];
            continue
        }
        if (args[i] === '-u' || args[i] === '--used') {
            usedAfter = new Date(0); // Epoch
            continue
        }
        if (args[i] === '-u' || args[i] === '--used-after') {
            usedAfter = new Date(args[++i]);
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
        await takeSnapshot(api, pallets, snapshotFile, gqlEndpoint);
    } else if (command === 'restore') {
        if (args.length !== 1) {
            usage();
        }
        const sudo = keyring.addFromUri(args[0]);
        await restore(api, keyring, pallets, snapshotFile, sudo, replace, usedAfter);
    } else {
        usage();
    }

    process.exit(0);
}

main().catch(console.error);

