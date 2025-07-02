// This script is executed on the test parachain to verify that:
// 1. the parachain received the teleport from the relay chain, minting tokens to the requested account
// 2. it is possible to request an XCM teleport of a given amount of tokens toward a given account on the relay chain
// 3. it is possible to request a custom remote execution on the relay chain through XCM (in this case a submitProof extrinsic)
// 4. the test parachain receives an XCM response indicating the outcome of the remote execution

const { BN } = require('@polkadot/util');

const { BLOCK_TIME, receivedEvents, submitExtrinsic, waitForEvent } = require('zkv-lib');

const ReturnCode = {
    Ok: 1,
    WrongTeleportReceived: 2,
    ExtrinsicUnsuccessful: 3,
    FailedRegisteringXcmResponse: 4,
    FailedSendingXcm: 5,
    TimeoutWaitingForXcmResponse: 6,
    RelayVerificationFailed: 7,
};

async function run(nodeName, networkInfo, args) {
    const {wsUri, userDefinedTypes} = networkInfo.nodesByName[nodeName];
    const api = await zombie.connect(wsUri, userDefinedTypes);

    // Alice's remote Computed Origin on the relay chain, computed offline with xcm-tools
    const ALICE_REMOTE_ORIGIN = '0x7b2ac6587a1931a0b108bb03777f8e552293bd6a6ea3790a5fe14e214f13072b';

    const keyring = new zombie.Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');

    const amount = args[0];
    const receiver = args[1];

    // 1. Check that we receive the teleport from the relay chain w/ the correct parameters

    console.log("Waiting for teleport from relay chain");

    let timeout = BLOCK_TIME * 3;
    let balance_receiver = (await api.query.system.account(receiver))["data"]["free"];

    while (!balance_receiver.eq(new BN(amount, 10))) {
        await new Promise(r => setTimeout(r, 1000));
        timeout -= 1000;
        balance_receiver = (await api.query.system.account(receiver))["data"]["free"];
        if (timeout <= 0) {
            console.log("Not yet received, giving up!");
            return ReturnCode.WrongTeleportReceived;
        }
    }

    console.log('Received balance: ' + balance_receiver.toHuman());

    // 2. Teleport to Alice's remote origin on relay chain

    const dest = {
        V4: {
            parents: '1',
            interior: {
                Here: '',
            },
        },
    };
    const beneficiary = {
        V4: {
            parents: '0',
            interior: {
                X1: [{
                    AccountId32: {
                        network: null,
                        id: ALICE_REMOTE_ORIGIN,
                    },
                }]
            },
        },
    };
    const assets = {
        V4: [{
                id: {
                    parents: 1,
                    interior: {
                        Here: '',
                    },
                },
                fun: {
                    Fungible: amount,
                },
        }],
    };

    const fee_asset_item = '0';
    const weight_limit = 'Unlimited';

    const teleport = await api.tx.xcmPallet.teleportAssets(dest, beneficiary, assets, fee_asset_item);

    console.log("Teleporting to Alice's remote origin on the relay chain");
    if (!receivedEvents(await submitExtrinsic(api, teleport, alice, BlockUntil.InBlock, undefined))) {
        console.log("Teleport failed!");
        return ReturnCode.ExtrinsicUnsuccessful;
    }

    // 3. Send an XCM message which includes a Transact instruction for verifying a groth16 proof

    const xcm_wait_for_response = await api.tx.xcmNotifications.prepareNewQuery();

    console.log("Registering for XCM response");
    const query_evts = (await submitExtrinsic(api, xcm_wait_for_response, alice, BlockUntil.InBlock,
      (event) => event.section == "xcmNotifications" && event.method == "QueryPrepared")).events;

    if (query_evts.length < 1) {
        return ReturnCode.FailedRegisteringXcmResponse;
    }
    const query_id = query_evts[0].data[0].toString();

    console.log("Registered for XCM response with id: " + query_id);

    // This is the asset that we use for remote execution.
    // Overestimating this should be ok, any surplus is refunded with the instructions in the appendix (see SetAppendix)
    const exec_amount = (new BN(amount, 10)).div(new BN(10, 10));
    const exec_asset = {
        id: {
            Concrete: {
                parents: 0,
                interior: {
                    Here: '',
                },
            },
        },
        fun: {
            Fungible: exec_amount,
        },
    }

    const instr_withdraw = {
        WithdrawAsset: [ exec_asset ],
    };

    const instr_buy_execution = {
        BuyExecution: {
            fees: exec_asset,
            weightLimit: { Unlimited: null },
        }
    };

    const response_cfg = {
        destination: {
            parents: '0',
            interior: {
                X1: [{ Parachain: 1599 }],
            },
        },
        queryId: query_id,
        maxWeight: {
            refTime: '1000000',
            proofSize: '65536',
        },
    }

    const instr_error_handler = {
        SetErrorHandler: [
            { ReportError: response_cfg }
        ]
    };

    const { CALL: TEST_CALL } = require(networkInfo["tmpDir"] + '/test_call.js')

    const instr_transact = {
        Transact: {
            originKind: { SovereignAccount: null },
            // This is the overestimated cost of the desired execution.
            // Any higher value should be ok, as long as it fits in a single block on the relay chain
            requireWeightAtMost: {
                refTime: '9000000000',
                proofSize: '177995',
            },
            call: TEST_CALL,
        }
    };

    const instr_report_transact_status = {
        ReportTransactStatus: response_cfg 
    };

    const instr_refund_surplus = {
        SetAppendix: [
            {
                RefundSurplus: null,
            },
            {
                DepositAsset: {
                    assets: { Wild: 'All' },
                    beneficiary: {
                        parents: 0,
                        interior: { X1: [{
                            AccountId32: {
                                network: null,
                                id: ALICE_REMOTE_ORIGIN,
                            },
                        }]}
                    }
                }
            }
        ]
    }

    // This is the actual XCM message, consisting of a vector of XCM instructions
    const remote_proof_verification = {
        V4: [instr_withdraw, instr_buy_execution, instr_refund_surplus, instr_error_handler, instr_transact, instr_report_transact_status]
    };

    const xcm_transact = await api.tx.xcmPallet.send(dest, remote_proof_verification);

    console.log("Sending XCM transact");
    const xcmsend_evts = (await submitExtrinsic(api, xcm_transact, alice, BlockUntil.InBlock,
      (event) => event.section == "xcmPallet" && event.method == "Sent")).events;

    if (query_evts.length < 1) {
        return ReturnCode.FailedSendingXcm;
    }

    // 4. Wait for the XCM response on the outcome of the remote execution

    console.log("Waiting for XCM response");
    const EXPECTED_RESP_TIMEOUT = BLOCK_TIME * 10; // 3 blocks for finalization on relay
                                                  // + 1 for response receival
                                                  // + 1 for XCM execution on para
    // + 6 as margin
    response = (await waitForEvent(api, EXPECTED_RESP_TIMEOUT, "xcmPallet", "ResponseReady"));

    if (response == -1) {
        return ReturnCode.TimeoutWaitingForXcmResponse;
    }

    if (response.data[0].toNumber() != query_id
        || response.data[1].toString() != '{"dispatchResult":{"success":null}}') {
        console.log(`Unexpected XCM response: qid=${response.data[0].toString()} (!= ${query_id}); data=${response.data[1].toString()}`);
        return ReturnCode.RelayVerificationFailed;
    }

    return ReturnCode.Ok;
}

module.exports = { run }
