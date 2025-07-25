// This script is executed on the relay chain to check that:
// 1. it is possible to request an XCM teleport of a given amount of tokens toward a given account on the test parachain
// 2. the cost of the teleport covers both the teleported tokens, and the XCM execution cost
//
// 3. The test also encodes a groth16 verification call, that will be executed remotely through XCM from the test parachain, on this chain.

const { BN, compactAddLength, u8aToHex } = require('@polkadot/util');

const fs = require('fs');

const { submitExtrinsic, receivedEvents } = require('zkv-lib');

const ReturnCode = {
    Ok: 1,
    WrongBalance: 2,
    ExtrinsicUnsuccessful: 3,
    FailedSavingFile: 4,
};

async function run(nodeName, networkInfo, args) {
    const {wsUri, userDefinedTypes} = networkInfo.nodesByName[nodeName];
    const api = await zombie.connect(wsUri, userDefinedTypes);

    const keyring = new zombie.Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');

    // Collect Alice's free balance
    const ALICE = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
    let balance_alice_pre = (await api.query.system.account(ALICE))["data"]["free"];
    console.log('Alice\'s balance: ' + balance_alice_pre.toHuman());

    const amount = args[0];
    const benef = args[1];
  
    // 1. Create an XCM teleport extrinsic, teleporting _amount_ tokens to _benef_
    const dest = {
        V4: {
            parents: '0',
            interior: {
              X1: [{ Parachain: 1599 }],
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
                    id: benef,
                  },
                }]
            },
        },
    };
    const assets = {
        V4: [{
            id: {
                    parents: 0,
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

    if (!receivedEvents(await submitExtrinsic(api, teleport, alice, BlockUntil.InBlock, undefined))) {
        return ReturnCode.ExtrinsicUnsuccessful;
    }

    // 2. Verify the cost of the teleport above

    // Get the updated balances
    balance_alice_post = (await api.query.system.account(ALICE))["data"]["free"];  
    console.log('Alice\'s balance after tx: ' + balance_alice_post.toHuman());

    let paid = balance_alice_pre.sub(balance_alice_post);

    if (paid.lte(new BN(amount, 10))) {
        console.log("Paid less than the teleport amount: " + paid.toString());
        return ReturnCode.WrongBalance;
    }

    return ReturnCode.Ok;
}

module.exports = { run }
