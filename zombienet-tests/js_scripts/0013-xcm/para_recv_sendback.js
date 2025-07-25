// This script is executed on the test parachain to verify that:
// 1. the parachain received the teleport from the relay chain, minting tokens to the requested account
// 2. it is possible to request an XCM teleport of a given amount of tokens toward a given account on the relay chain
// 3. it is possible to request a custom remote execution on the relay chain through XCM (in this case a submitProof extrinsic)
// 4. the test parachain receives an XCM response indicating the outcome of the remote execution

const { BN } = require('@polkadot/util');

const { BLOCK_TIME, receivedEvents, submitExtrinsic } = require('zkv-lib');

const ReturnCode = {
    Ok: 1,
    WrongTeleportReceived: 2,
    ExtrinsicUnsuccessful: 3,
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

    return ReturnCode.Ok;
}

module.exports = { run }
