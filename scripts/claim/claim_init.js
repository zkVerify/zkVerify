const { ApiPromise, WsProvider } = require('@polkadot/api');
const Keyring = require('@polkadot/keyring').default;
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
const DEFAULT_BATCH_SIZE = 10000;
const DEFAULT_PREFIX = 251;

// Claim Manager Account
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

async function read_airdrop_csv(csv_file, address_col, amount_col, batch_size) {
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
        amount = parse_amount(amount);

        if (BigInt(amount) == 0) return;

        address = entry[address_col];
        if (address == undefined) {
            throw new Error(`Cannot find any address in column ${address_col}`);
        }
        if (current_map.has(address)) {
            throw new Error("Duplicate address found!");
        };
        if (USE_PALLET_TOKEN_CLAIM) {
            const type = address.startsWith("0x") ? `Ethereum` : `Substrate`;
            if (type == "Ethereum" && address.length != 42
                || type == "Substrate" && address.length != 49) {
                throw new Error(`Unexpected format for ${type} address: ${address}`);
            }
            current_map.set({[`${type}`]: address}, amount);
        } else {
            current_map.set(address, amount);
        }
        count++;
        if (count % batch_size == 0) {
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

async function send_as_multi_sudo(api, account, threshold, signatories, call) {
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

    return {
           "block": send_result.number,
           "index": send_result.index,
           "hash": send_result.call,
           "weight": (await sudo_call.paymentInfo(account)).weight,
           "data": sudo_call.method.toHex()};
}

async function send_as_sudo(api, account, call) {
    const sudo_call = api.tx.sudo.sudo(call);

    const sendFunction = (callback) => sudo_call.signAndSend(account, callback);

    return await _handleTransactionLifecycle(api, sendFunction);
}

async function send(api, account, threshold, signatories, call) {
    if (threshold > 0) {
        return send_as_multi_sudo(api, account, threshold, signatories, call);
    }
    return send_as_sudo(api, account, call);
}

async function send_begin_claim_tx(api, account, threshold, signatories, beneficiaries, msg) {
    let begin_claim_tx = undefined;
    if (USE_PALLET_TOKEN_CLAIM) {
        begin_claim_tx = api.tx.tokenClaim.beginClaim(beneficiaries, msg);
    } else {
        begin_claim_tx = api.tx.claim.beginAirdrop(beneficiaries);
    }
    return send(api, account, threshold, signatories, begin_claim_tx);
}

async function send_add_beneficiaries_tx(api, account, threshold, signatories, beneficiaries) {
    let add_claim_tx = undefined;
    if (USE_PALLET_TOKEN_CLAIM) {
        add_claim_tx = api.tx.tokenClaim.addBeneficiaries(beneficiaries);
    } else {
        add_claim_tx = api.tx.claim.addBeneficiaries(beneficiaries);
    }
    return send(api, account, threshold, signatories, add_claim_tx);
}

async function check_preconditions(api, beneficiaries, batch_size) {
    let claim_active   = undefined;
    let free_balance   = undefined;
    let max_batch_size = 10000; // this is the default for the old pallet

    // fetch data
    if (USE_PALLET_TOKEN_CLAIM) {
        claim_active = await api.query.tokenClaim.claimActive();
        max_batch_size = await api.consts.tokenClaim.maxOpBeneficiaries.toNumber();
        const pallet_address = await api.query.tokenClaim.palletAccountId();
        free_balance = (await api.query.system.account(String(pallet_address)))["data"]["free"];
    } else {
        claim_active = await api.query.claim.airdropActive();
        const pallet_address = await api.query.claim.palletAccountId();
        free_balance = (await api.query.system.account(String(pallet_address)))["data"]["free"];
    }

    // check state
    if (claim_active == INIT_CAMPAIGN) {
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

    if (batch_size > max_batch_size) {
        console.log(`Batch size ${batch_size} exceeds the maximum supported ${max_batch_size}`);
        return false;
    }

    return true;
}

function usage() {
    console.error(`Usage:
    node claim_init.js [OPTIONS] csv_input_file

    OPTIONS:
    -b, --batch-size: max num of addresses for a single extrinsic | ${DEFAULT_BATCH_SIZE}
    -e, --end-point: url or key from ${Object.keys(DEFAULT_WS_ENDPOINTS)} | ${DEFAULT_WS_ENDPOINT}
    -h, --help: show this help and exit
    -k, --claim-msg: claim message to pass to the beginClaim extrinsic (only for pallet tokenClaim) | ${DEFAULT_CLAIM_MSG_PREFIX}
    -m, --multi-address: address of the other signatories of the multisig account (can be repeated to add multiple addresses) | ${DEFAULT_MULTISIG_ADDRESSES}
    -n, --multi-threshold: threshold for the multisig account | ${DEFAULT_MULTISIG_THRESHOLD}
    -o, --out: path where multisig call data are stored | ${DEFAULT_OUT_FILE_PATH}
    -p, --prefix: the prefix for ss58 addresses on the chain | ${DEFAULT_PREFIX}
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
        ss58_prefix: DEFAULT_PREFIX,
        batch_size: DEFAULT_BATCH_SIZE,
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
        else if (args[i] === '-p' || args[i] === '--prefix') {
            result.ss58_prefix = args[++i];
            continue;
        }
        else if (args[i] === '-b' || args[i] === '--batch-size') {
            result.batch_size = args[++i];
            continue;
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
        || result.batch_size <= 0
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

function get_approve_as_multi(api, approvee, threshold, approver, signatories) {
    return api.tx.multisig.approveAsMulti(
        threshold,
        signatories.filter(key => key != approver).sort(),
        { "height": approvee.block, "index": approvee.index },
        approvee.hash,
        0
    );
}

function get_approve_batch(api, approvees, threshold, approver, signatories) {
    let calls = [];
    approvees.forEach(a => {
        calls.push(get_approve_as_multi(api, a, threshold, approver, signatories));
    });
    return api.tx.utility.batchAll(calls);
}

function get_final_approve(api, approvee, threshold, approver, signatories) {
    return api.tx.multisig.asMulti(
        threshold,
        signatories.filter(key => key != approver).sort(),
        { "height": approvee.block, "index": approvee.index },
        approvee.data,
        approvee.weight
    );
}

function check_no_prev_calls(out_path, addrs) {
    for (addr of addrs) {
        const path = `${out_path}/${addr}`;
        if (fs.existsSync(path)) {
            console.log(`Out dir ${path} already exists, please delete that before proceeding`);
            return false;
        }
    }
    return true;
}

function write_new_file_name(path, fname_prefix, addr, data) {
    fs.mkdirSync(path, { recursive: true });
    // Find a non-existing filename
    fname_index = 0;
    fname = "";
    do {
        fname = `${path}/${fname_prefix}_${addr}_${fname_index}.txt`;
        ++fname_index;
    } while (fs.existsSync(fname));

    fs.writeFileSync(fname, data);
    return fname;
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

    if (!check_no_prev_calls(options.out_file_path, options.multisig_addresses)) {
        print_error("Found leftovers from previous runs; quitting");
        process.exit(1);
    }

    const beneficiaries = await read_airdrop_csv(options.csv_file,
                                                  options.address_col_label,
                                                  options.amount_col_label,
                                                  options.batch_size);

    console.log(`Number of batches: ${beneficiaries.length}`);

    // Create a WebSocket provider
    const wsProvider = new WsProvider(options.ws_endpoint);

    // Initialize the API
    const api = await ApiPromise.create({ provider: wsProvider });

    if (!await check_preconditions(api, beneficiaries, options.batch_size)) {
        print_error("Some preconditions are not met; bailing out");
        wsProvider.disconnect();
        return;
    }

    const keyring = new Keyring({ type: 'sr25519', ss58Format: options.ss58_prefix });
    const account = keyring.addFromUri(options.secret);

    multi_calls_to_be_approved = [];
    for (let m = 0; m < beneficiaries.length; m++) {
        print_highlight(`Sending chunk ${m}`);
        let result = 0;
        if (INIT_CAMPAIGN && m == 0) {
            result = await send_begin_claim_tx(api,
                                               account,
                                               options.threshold,
                                               options.multisig_addresses,
                                               beneficiaries[m],
                                               options.claim_msg);
        }
        else {
            result = await send_add_beneficiaries_tx(api,
                                                     account,
                                                     options.threshold,
                                                     options.multisig_addresses,
                                                     beneficiaries[m]);
        }
        if (result === -1) {
            print_error("Something went wrong with the extrinsic; stop sending");
            break;
        }

        if (options.threshold > 0) {
            multi_calls_to_be_approved.push(result);
        }
    }

    console.log(`Have ${multi_calls_to_be_approved.length} multi calls to be approved`);

    for (addr of options.multisig_addresses) {
        const path = `${options.out_file_path}/${addr}`;
        // batched calls for intermediate approvals
        approve_call_hex = get_approve_batch(api,
                                     multi_calls_to_be_approved,
                                     options.threshold,
                                     addr,
                                     options.multisig_addresses);

        fname = write_new_file_name(path,
                                    "encoded_batch_approve",
                                    addr,
                                    approve_call_hex.toHex());

        // calls for final approve
        for (m of multi_calls_to_be_approved) {
            final_approve_call = get_final_approve(api,
                                                   m,
                                                   options.threshold,
                                                   addr,
                                                   options.multisig_addresses);

            fname = write_new_file_name(path,
                                        "encoded_final_approve",
                                        addr,
                                        final_approve_call.toHex());

        }

        print_highlight(`Written encoded calls to ${path} <== SHARE THIS DIR WITH ${addr} FOR APPROVAL`);
    }

    print_highlight(`All done, closing the connection!`);
    wsProvider.disconnect();
}

main().catch(console.error);
