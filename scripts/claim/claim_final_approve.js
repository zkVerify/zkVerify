const { ApiPromise, WsProvider } = require('@polkadot/api');
const { isHex } = require('@polkadot/util');
const Keyring = require('@polkadot/keyring').default;
const { BigInt } = require('@polkadot/x-bigint');
const fs = require('fs');
const readline = require('readline');

// CHAIN INFO -- dev
const DEFAULT_WS_ENDPOINT = 'ws://localhost:9944';
const DEFAULT_WS_ENDPOINTS = {
    'local': DEFAULT_WS_ENDPOINT,
    'volta': 'wss://zkverify-volta-rpc.zkverify.io',
    'mainnet': 'wss://zkverify-rpc.zkverify.io',
}
const DEFAULT_PREFIX = 251;

// Claim Manager Account
const DEFAULT_SEED = '//Alice';

const DEFAULT_OUT_FILE_PATH = '.';

const INITIAL_CONFIRMATION_MSG =
`
This script will submit extrinsics - incurring transacion fees - from the selected account.
Notice, the selected account MUST be the manager account for the claim pallet.
If you use a multisig account that is not the manager for the claim pallet, this script will succeed, but the extrinsics will fail upon approval from the other signatories.

Please review the parameters above, and confirm if you want to proceed (Y/N).
`;


function print_error(msg) {
    console.error('\x1b[31m%s\x1b[0m', msg);
}

function print_highlight(msg) {
    console.error('\x1b[32m%s\x1b[0m', msg);
}

async function _handleTransactionLifecycle(api, sendFunction) {
    let transactionSuccessEvent = false;
    let done = false;
    let max_retries = 5;
  
    let retVal = -1;
    while (!done && max_retries > 0) {
        retVal = await new Promise(async (resolve, reject) => {
            const unsub = await sendFunction(({ blockNumber, events: records = [], status }) => {
                let blockHash = null;
                let callHash = null;
                let indexInBlock = null;
                if (status.isInBlock) {
                    blockHash = status.asInBlock;
                    console.log(`Transaction included at blockhash ${blockHash}`);
                    records.forEach(({ event: { method, section, data }, phase }) => {
                        if (section == "system" && method == "ExtrinsicSuccess") {
                            transactionSuccessEvent = true;
                        }
                        else if (section == "multisig" && method == "NewMultisig") {
                            callHash = data[2];
                            indexInBlock = phase.asApplyExtrinsic;
                        }
                    });
                    done = true;
                  }
                  else if (status.isError) {
                      done = true;
                      console.log("ERROR: Transaction status.isError");
                  }
                  if (done) {
                      unsub();
                      if (transactionSuccessEvent) {
                          resolve([blockHash, callHash, blockNumber, indexInBlock]);
                      } else {
                          reject("ExtrinsicSuccess has not been seen");
                      }
                  }
                })  .catch(
                    error => {
                        console.log(`Sending extrinsic failed with error: ${error}`);
                        if (error.code === 1014) { // priority too low error
                            reject("retry");
                        } else {
                            reject(error);
                        }
                    }
                );
          })  .then(
              ([blockHash, callHash, blockNumber, indexInBlock]) => {
                console.log(`Transaction successfully processed [${blockHash}]`);
                return {
                  block: blockHash,
                  number: blockNumber,
                  index: indexInBlock,
                  call: callHash,
                };
              },
              async function (error) {
                if (error !== "retry") {
                  console.log("Not retrying!");
                  done = true;
                  return -1;
                }
                console.log("Retrying");
                --max_retries;
            }
        );
    }
  
    return retVal;
}

async function send(api, account, call) {
    if (!isHex(call)) {
        console.log(`Call is not hex encoded!`);
        return -1;
    }
    const sendFunction = (callback) => api.tx(call).signAndSend(account, callback);

    return await _handleTransactionLifecycle(api, sendFunction);
}

function usage() {
    console.error(`Usage:
    node claim_init.js [OPTIONS] csv_input_file

    OPTIONS:
    -e, --end-point: url or key from ${Object.keys(DEFAULT_WS_ENDPOINTS)} | ${DEFAULT_WS_ENDPOINT}
    -h, --help: Show this help and exist
    -o, --out: path where multisig call data are stored | ${DEFAULT_OUT_FILE_PATH}
    -p, --prefix: the prefix for ss58 addresses on the chain | ${DEFAULT_PREFIX}
    -u, --uri: secret seed to sign the extrinsics | ${DEFAULT_SEED}

    EXAMPLE (multisig address [Alice, Bob, Charlie] with 2/3 threshold, proposed by Charlie):
    node claim_final_approve.js -n 2 -m xpjztNMwdaEqGLJ6sH6p7WTnuyHHroUAw9HELRxdPNzKr2Efi -m xpiRj6HmVSWqNBHcMrArAm7swsSGyLvmbeT61NbViK1QcFMqY -m xpiUPATuXmWizou22NxSVknLEhZxMLEeT5EMLubnpbQb8YWhU --uri //Charlie
    `);
    process.exit(1);
}

function parse_args() {
    let args = process.argv.slice(2)
    let result = {
        ws_endpoint: DEFAULT_WS_ENDPOINT,
        ss58_prefix: DEFAULT_PREFIX,
        secret: DEFAULT_SEED,
        out_file_path: DEFAULT_OUT_FILE_PATH,
    }

    // Skipping command and remove general options like -e
    const extra_args = [];
    for (let i = 0; i < args.length; i += 1) {
        if (args[i] === '-e' || args[i] === '--end-point') {
            result.ws_endpoint = args[++i];
            let address = DEFAULT_WS_ENDPOINTS[result.ws_endpoint];
            if (address !== undefined) {
                result.ws_endpoint = address
            }
            continue
        }
        else if (args[i] === '-p' || args[i] === '--prefix') {
            result.ss58_prefix = args[++i];
            continue;
        }
        else if (args[i] === '-u' || args[i] === '--uri') {
            result.secret = args[++i] 
            continue;
        }
        else if (args[i] === '-o' || args[i] === '--out') {
            result.out_file_path = args[++i];
            continue
        }
        else if (args[i] === '-h' || args[i] === '--help') {
            usage();
        }
        extra_args.push(args[i]);
    }
    if (extra_args.length != 0) {
        usage();
    }
    return result;
}

function confirmation(query) {
    const rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout,
    });

    return new Promise(resolve => rl.question(query, ans => {
        rl.close();
        resolve(ans);
    }))
}

function read_file_name(path, fname_prefix, addr, idx) {
    addr_path = `${path}/${addr}`;
    fname = `${addr_path}/${fname_prefix}_${addr}_${idx}.txt`;
    if (!fs.existsSync(fname)) {
        return undefined;
    }

    return `${fs.readFileSync(fname)}`;
}

function read_files(path, fname_prefix, addr) {
    txs = [];
    idx = 0;
    tx = read_file_name(path, fname_prefix, addr, idx);
    while (tx != undefined) {
        txs.push(tx);
        tx = read_file_name(path, fname_prefix, addr, ++idx);
    }
    return txs;
}

async function main() {
    const options = parse_args();
    console.log(`USING PARAMETERS:`);
    for (const [k, v] of Object.entries(options)) {
        console.log(`${k}: ${v}`);
    }

    let proceed = await confirmation(INITIAL_CONFIRMATION_MSG);

    if (proceed !== "Y" && proceed !== "y") {
        process.exit(0);
    }

    // Create a WebSocket provider
    const wsProvider = new WsProvider(options.ws_endpoint);

    // Initialize the API
    const api = await ApiPromise.create({ provider: wsProvider });

    const keyring = new Keyring({ type: 'sr25519', ss58Format: options.ss58_prefix });
    const account = keyring.addFromUri(options.secret);

    const txs = read_files(options.out_file_path, "encoded_final_approve", account.address);

    console.log(`Number of extrinsics: ${txs.length}`);

    i = 0;
    for (tx of txs) {
        print_highlight(`Sending tx ${i++}`);
        const result = await send(api, account, tx);
        if (result === -1) {
            print_error("Something went wrong with the extrinsic; stop sending");
            break;
        }
    }

    wsProvider.disconnect();
}

main().catch(console.error);
