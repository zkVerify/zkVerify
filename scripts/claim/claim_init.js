const { ApiPromise, WsProvider } = require('@polkadot/api');
const Keyring = require('@polkadot/keyring').default;
const { BN } = require('@polkadot/util');
const { BigInt } = require('@polkadot/x-bigint');
const fs = require('fs');
const neatCsv = require('neat-csv').default;
const readline = require('readline');

// CSV FILE
const DEFAULT_ADDRESS_COL = 'address';
const DEFAULT_AMOUNT_COL = 'total';

// CHAIN INFO -- dev
const DEFAULT_WS_ENDPOINT = 'ws://localhost:9944';
const DEFAULT_WS_ENDPOINTS = {
    'local': DEFAULT_WS_ENDPOINT,
    'volta': 'wss://zkverify-volta-rpc.zkverify.io',
    'mainnet': 'wss://zkverify-rpc.zkverify.io',
}
const EXISTENTIAL_DEPOSIT = '10000000000000000';
const MAX_PER_BLOCK = 11000;

// SUDO ACCOUNT
const DEFAULT_SEED = '//Alice';
const DEFAULT_MULTISIG_THRESHOLD = 0;
const DEFAULT_MULTISIG_ADDRESSES = []

const DEFAULT_OUT_FILE_PATH = '.';

const INIT_CAMPAIGN = true;
const USE_PALLET_TOKEN_CLAIM = true;
const DEFAULT_CLAIM_MSG_PREFIX = 'zkverify claim';

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

function parse_amount(tokens) {
    multiplier = 18;
    decimal_position = tokens.indexOf('.');
    if (decimal_position != -1)
        multiplier = multiplier - (tokens.length - 1 - decimal_position);
    tokens = tokens.replace('.', '').replace(/^0+/, '');
    for (let i = 0; i < multiplier; i++) {
        tokens += '0';
    }
    return tokens;
}

async function read_airdrop_csv(csv_file, address_col, amount_col) {
    const fdata = fs.readFileSync(csv_file);
    const csv_data = await neatCsv(fdata);
    beneficiaries = [];
    count = 0;
    current_map = new Map();
    csv_data.forEach(entry => {
        amount = entry[amount_col];
        if (amount == undefined) {
            throw new Error(`Cannot find any amount in column ${amount_col}`);
        }
        if (BigInt(amount) == 0) return;

        address = entry[address_col];
        if (address == undefined) {
            throw new Error(`Cannot find any address in column ${address_col}`);
        }
        if (current_map.has(address)) {
            throw new Error("Duplicate address found!");
        };
        if (USE_PALLET_TOKEN_CLAIM) {
            // TODO: DO NOT HARDCODE THIS!
            current_map.set({'Substrate': address}, parse_amount(amount));
        } else {
            current_map.set(address, parse_amount(amount));
        }
        count++;
        if (count % MAX_PER_BLOCK == 0) {
            beneficiaries.push(current_map);
            current_map = new Map();
        }
    });
    beneficiaries.push(current_map);
    console.log(`Read ${count} non-zero beneficiaries`);
    return beneficiaries;
}

async function _handleTransactionLifecycle(api, sendFunction) {
    let transactionSuccessEvent = false;
    let done = false;
    let max_retries = 5;
  
    let retVal = -1;
    while (!done && max_retries > 0) {
        retVal = await new Promise(async (resolve, reject) => {
            const unsub = await sendFunction(({ events: records = [], status }) => {
                let blockHash = null;
                let callHash = null;
                if (status.isInBlock) {
                    blockHash = status.asInBlock;
                    console.log(`Transaction included at blockhash ${blockHash}`);
                    records.forEach(({ event: { method, section, data } }) => {
                        if (section == "system" && method == "ExtrinsicSuccess") {
                            transactionSuccessEvent = true;
                        }
                        else if (section == "multisig" && method == "NewMultisig") {
                            callHash = data[2];
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
                          resolve([blockHash, callHash]);
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
              ([blockHash, callHash]) => {
                console.log(`Transaction successfully processed [${blockHash}]`);
                return {
                  block: blockHash,
                  call: callHash,
                };
              },
              async function (error) {
                if (error !== "retry") {
                  console.log("Not retrying!");
                  return -1;
                }
                console.log("Retrying");
                ++max_retries;
            }
        );
    }
  
    return retVal;
}

async function send_as_multi_sudo(api, account, threshold, signatories, call, fname) {
    const sudo_call = api.tx.sudo.sudo(call);
    const proposal = api.tx.multisig.asMulti(
        threshold,
        signatories.filter(key => key != account.address).sort(),
        null,
        sudo_call.method.toHex(),
        0
    );

    const sendFunction = (callback) => proposal.signAndSend(account, callback);

    let send_result = await _handleTransactionLifecycle(api, sendFunction);

    if (send_result === -1) {
        return send_result;
    }

    const data = `CALL_HASH: ${send_result.call}\nCALL_DATA: ${sudo_call.method.toHex()}`;
    fs.writeFileSync(fname, data);
}

async function send_as_sudo(api, account, call) {
    const sudo_call = api.tx.sudo.sudo(call);

    const sendFunction = (callback) => sudo_call.signAndSend(account, callback);

    return await _handleTransactionLifecycle(api, sendFunction);
}

async function send(api, account, threshold, signatories, call, fname) {
    if (threshold > 0) {
        return send_as_multi_sudo(api, account, threshold, signatories, call, fname);
    }
    return send_as_sudo(api, account, call);
}

async function send_begin_claim_tx(api, account, threshold, signatories, beneficiaries, msg, fname) {
    let begin_claim_tx = undefined;
    if (USE_PALLET_TOKEN_CLAIM) {
        begin_claim_tx = api.tx.tokenClaim.beginClaim(beneficiaries, msg);
    } else {
        begin_claim_tx = api.tx.claim.beginAirdrop(beneficiaries);
    }
    return send(api, account, threshold, signatories, begin_claim_tx, fname);
}

async function send_add_beneficiaries_tx(api, account, threshold, signatories, beneficiaries, fname) {
    let add_claim_tx = undefined;
    if (USE_PALLET_TOKEN_CLAIM) {
        add_claim_tx = api.tx.tokenClaim.addBeneficiaries(beneficiaries);
    } else {
        add_claim_tx = api.tx.claim.addBeneficiaries(beneficiaries);
    }
    return send(api, account, threshold, signatories, add_claim_tx, fname);
}

async function check_preconditions(api, beneficiaries) {
    let claim_active = undefined;
    let free_balance = undefined;

    // fetch data
    if (USE_PALLET_TOKEN_CLAIM) {
        claim_active = await api.query.claim.airdropActive();
        const pallet_address = await api.query.tokenClaim.palletAccountId();
        free_balance = (await api.query.system.account(String(pallet_address)))["data"]["free"];
    } else {
        claim_active = await api.query.tokenClaim.airdropActive();
        const pallet_address = await api.query.claim.palletAccountId();
        free_balance = (await api.query.system.account(String(pallet_address)))["data"]["free"];
    }

    // check state
    if (claim_active === INIT_CAMPAIGN) {
        console.log("A claim is already active on chain!");
        return false;
    }

    // check balance
    tot = BigInt(EXISTENTIAL_DEPOSIT);
    for (let m = 0; m < beneficiaries.length; m++) {
        for (let [k, v] of beneficiaries[m]) {
            tot += BigInt(v);
        }
    }

    if (free_balance < tot) {
        console.log(`Not enough tokens!`);
        console.log(`Need at least ${tot}`);
        console.log(`free_balance ${free_balance}`);
        return false;
    }
    return true;
}

function usage() {
    console.error(`Usage:
    node claim_init.js [OPTIONS] csv_input_file

    OPTIONS:
    -e, --end-point: url or key from ${Object.keys(DEFAULT_WS_ENDPOINTS)} | ${DEFAULT_WS_ENDPOINT}
    -h, --help: Show this help and exist
    -k, --claim-msg: claim message to pass to the beginClaim extrinsic (only for pallet tokenClaim) | ${DEFAULT_CLAIM_MSG_PREFIX}
    -m, --multi-address: address of the other signatories of the multisig account (can be repeated to add multiple addresses) | ${DEFAULT_MULTISIG_ADDRESSES}
    -n, --multi-threshold: threshold for the multisig account | ${DEFAULT_MULTISIG_THRESHOLD}
    -o, --out: path where multisig call data are stored | ${DEFAULT_OUT_FILE_PATH}
    -s, --col-address: name of the column representing the beneficiary address in the csv file | ${DEFAULT_ADDRESS_COL}
    -t, --col-amount: name of the column representing the amount for a beneficiary in the csv file | ${DEFAULT_AMOUNT_COL}
    -u, --uri: secret seed to sign the extrinsics | ${DEFAULT_SEED}

    EXAMPLE (multisig address [Alice, Bob, Charlie] with 2/3 threshold, proposed by Charlie):
    node claim_init.js claim_data.csv -n 2 -m xpjztNMwdaEqGLJ6sH6p7WTnuyHHroUAw9HELRxdPNzKr2Efi -m xpiRj6HmVSWqNBHcMrArAm7swsSGyLvmbeT61NbViK1QcFMqY -m xpiUPATuXmWizou22NxSVknLEhZxMLEeT5EMLubnpbQb8YWhU --uri //Charlie
    `);
    process.exit(1);
}

function parse_args() {
    let args = process.argv.slice(2)
    let result = {
        csv_file: undefined,
        ws_endpoint: DEFAULT_WS_ENDPOINT,
        address_col_label: DEFAULT_ADDRESS_COL,
        amount_col_label: DEFAULT_AMOUNT_COL,
        secret: DEFAULT_SEED,
        threshold: DEFAULT_MULTISIG_THRESHOLD,
        multisig_addresses: DEFAULT_MULTISIG_ADDRESSES,
        out_file_path: DEFAULT_OUT_FILE_PATH,
        claim_msg: `${DEFAULT_CLAIM_MSG_PREFIX} ${Date.now()}`,
    }

    // Skipping command and remove general options like -e
    const extra_args = [];
    let is_multisig = false;
    for (let i = 0; i < args.length; i += 1) {
        if (args[i] === '-e' || args[i] === '--end-point') {
            result.ws_endpoint = args[++i];
            let address = DEFAULT_WS_ENDPOINTS[result.ws_endpoint];
            if (address !== undefined) {
                result.ws_endpoint = address
            }
            continue
        }
        else if (args[i] === '-k' || args[i] === '--claim-msg') {
            result.claim_msg = args[++i];
            continue
        }
        else if (args[i] === '-u' || args[i] === '--uri') {
            result.secret = args[++i] 
            continue;
        }
        else if (args[i] === '-m' || args[i] === '--multi-address') {
            if (!is_multisig) {
                is_multisig = true;
                result.multisig_addresses = [];
            }
            result.multisig_addresses.push(args[++i]);
            continue;
        }
        else if (args[i] === '-n' || args[i] === '--multi-threshold') {
            result.threshold = args[++i];
            continue;
        }
        else if (args[i] === '-o' || args[i] === '--out') {
            result.out_file_path = args[++i];
            continue
        }
        else if (args[i] === '-s' || args[i] === '--col-address') {
            result.address_col_label = args[++i];
            continue
        }
        else if (args[i] === '-t' || args[i] === '--col-amount') {
            result.amount_col_label = args[++i];
            continue
        }
        else if (args[i] === '-h' || args[i] === '--help') {
            usage();
        }
        extra_args.push(args[i]);
    }
    if (extra_args.length != 1
        || result.multisig_addresses.length < result.threshold
        || result.multisig_addresses.length && (result.threshold == 0)
        ) {
        usage();
    }
    result.csv_file = extra_args[0];
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

    const beneficiaries = await read_airdrop_csv(options.csv_file,
                                                  options.address_col_label,
                                                  options.amount_col_label);

    console.log(`Number of batches: ${beneficiaries.length}`);

    // Create a WebSocket provider
    const wsProvider = new WsProvider(options.ws_endpoint);

    // Initialize the API
    const api = await ApiPromise.create({ provider: wsProvider });

    if (!await check_preconditions(api, beneficiaries)) {
        print_error("Some preconditions are not met; bailing out");
        wsProvider.disconnect();
        return;
    }

    const keyring = new Keyring({ type: 'sr25519', ss58Format: 251 });
    const account = keyring.addFromUri(options.secret);

    for (let m = 0; m < beneficiaries.length; m++) {
        let fname = `${options.out_file_path}/call_data_${m}.txt`;
        print_highlight(`Sending chunk ${m}`);
        if (options.threshold > 0) {
            print_highlight(`Writing call data to ${fname}; <== SHARE THIS FILE WITH THE OTHER SIGNATORIES FOR APPROVAL`);
        }
        let result = 0;
        if (INIT_CAMPAIGN && m == 0) {
            result = await send_begin_claim_tx(api,
                                               account,
                                               options.threshold,
                                               options.multisig_addresses,
                                               beneficiaries[m],
                                               options.claim_msg,
                                               fname);
        }
        else {
            result = await send_add_beneficiaries_tx(api,
                                                     account,
                                                     options.threshold,
                                                     options.multisig_addresses,
                                                     beneficiaries[m],
                                                     fname);
        }
        if (result === -1) {
            print_error("Something went wrong with the extrinsic; bailing out");
            process.exit(-1);
        }
    }

    wsProvider.disconnect();
}

main().catch(console.error);
