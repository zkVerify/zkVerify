const util = zombie.util;
const { submitExtrinsic, BlockUntil } = require('zkv-lib')
const ReturnCode = {
    Ok: 1,
    ErrWrongSudoAccount: 2,
    ErrClaimMembership: 3,
    ErrSameRuntimeVersion: 4,
    ErrUnsupportedNetwork: 5,
};
const fs = require('fs');

async function run(nodeName, networkInfo, args) {
    const { wsUri, userDefinedTypes } = networkInfo.nodesByName[nodeName];
    const api = await zombie.connect(wsUri, userDefinedTypes);

    const [chain, realNodeName, nodeVersion] = await Promise.all([
        api.rpc.system.chain(),
        api.rpc.system.name(),
        api.rpc.system.version()
    ]);

    console.log(`Connected to chain: ${chain}`);
    console.log(`Node name: ${realNodeName}`);
    console.log(`Node version: ${nodeVersion}`);

    const ALICE = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
    const BOB = '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty';
    const CHARLIE = '5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y';

    // Use the keyring to generate accounts
    const keyring = new zombie.Keyring({ type: 'sr25519' });
    const alice = keyring.addFromUri('//Alice');
    const bob = keyring.addFromUri('//Bob');
    const charlie = keyring.addFromUri('//Charlie');

    /*****************************************************************************************************
     *************************************** CREATE MULTISIG ACCOUNT *************************************
     *****************************************************************************************************/
    const threshold = 2;
    const multisigAddress = util.createKeyMulti([alice.address, bob.address, charlie.address], threshold);
    const SS58Prefix = 251;
    const Ss58MultiAddress = util.encodeAddress(multisigAddress, SS58Prefix);
    console.log(`multisigAddress ${Ss58MultiAddress}`);


    /*****************************************************************************************************
     *************************************** SET MULTISIG AS SUDO ACCOUNT ********************************
     *****************************************************************************************************/

    const newSudoKey = api.tx.sudo.setKey(Ss58MultiAddress);
    await submitExtrinsic(api, newSudoKey, alice, BlockUntil.Finalized, undefined);

    const newSudoKeyOption = await api.query.sudo.key();
    if (newSudoKeyOption.isSome && newSudoKeyOption.unwrap().toString() !== Ss58MultiAddress) {
        return ReturnCode.ErrWrongSudoAccount;
    }

    /*****************************************************************************************************
     *************************************** GET CURRENT RUNTIME VERSION******* **************************
     *****************************************************************************************************/

    const currentRuntimeVersion = (await api.rpc.state.getRuntimeVersion()).specVersion.toNumber();

    /*****************************************************************************************************
     *************************************** SCHEDULE RUNTIME UPGRADE*************************************
     *****************************************************************************************************/

    // Retrieve the runtime to upgrade
    const sudoAccount = await api.query.sudo.key()
    let wasm;
    if (chain.toString().startsWith("Volta ")) {
        wasm = fs.readFileSync('./new-runtime/volta_runtime.wasm');
    } else if (chain.toString().startsWith("Zkverify ")) {
        wasm = fs.readFileSync('./new-runtime/zkv_runtime.wasm');
    } else {
        console.log(`Unsupported chain ${chain}, only Volta and Zkverify are supported`);
        return ReturnCode.ErrUnsupportedNetwork;
    }

    const code = wasm.toString('hex');
    const updateRuntimeCall = api.tx.system.setCode(`0x${code}`);

    console.log(`Upgrading from ${sudoAccount}, ${code.length / 2} bytes`);

    const { block } = await api.rpc.chain.getBlock();
    const blockNumber = block.header.number.toNumber();

    const when = blockNumber + 5;
    const priority = 100;

    const scheduleTx = api.tx.scheduler.schedule(when, null, priority, updateRuntimeCall);
    const sudoScheduleTx = api.tx.sudo.sudo(scheduleTx)

    const paymentInfo = await sudoScheduleTx.paymentInfo(alice);

    const proposal = api.tx.multisig.asMulti(
        threshold,
        [BOB, CHARLIE].sort(),
        null,
        sudoScheduleTx.method.toHex(),
        0
    );

    await submitExtrinsic(api, proposal, alice, BlockUntil.InBlock, undefined);

    const info = await api.query.multisig.multisigs(
        multisigAddress,
        sudoScheduleTx.method.hash
    );

    const approval = api.tx.multisig.asMulti(
        threshold,
        [ALICE, CHARLIE].sort(),
        info.unwrapOr(null).when,
        sudoScheduleTx.method.toHex(),
        paymentInfo.weight
    );

    await submitExtrinsic(api, approval, bob, BlockUntil.InBlock, undefined);

    const { block: currentBlockNumber } = await api.rpc.chain.getBlock();
    const blockNumberEnd = currentBlockNumber.header.number.toNumber();
    console.log("Scheduled upgrade expected to happen at block height " + when);
    console.log("Current block height after Multisig was submitted " + blockNumberEnd);

    /*****************************************************************************************************
     **************************** GENERATE BLOCKS UNTIL SCHEDULED CALL EXECUTED***************************
     *****************************************************************************************************/

    const blocksToWait = when - blockNumberEnd + 1;
    console.log(`Waiting for ${blocksToWait} blocks to be produced...`);

    await waitForBlocks(api, blocksToWait);

    /*****************************************************************************************************
     **************************** FINAL TEST: CHECK RUNTIME VERSION CHANGED*******************************
     *****************************************************************************************************/

    const updatedRuntimeVersion = (await api.rpc.state.getRuntimeVersion()).specVersion.toNumber();

    if (updatedRuntimeVersion === currentRuntimeVersion) {
        return ReturnCode.ErrSameRuntimeVersion
    }

    return ReturnCode.Ok;
}

async function waitForBlocks(api, numberOfBlocks) {
    return new Promise((resolve, reject) => {
        let startingBlockNumber;
        let currentBlockNumber;

        api.rpc.chain.subscribeNewHeads(async (header) => {
            if (!startingBlockNumber) {
                startingBlockNumber = header.number.toNumber();
                currentBlockNumber = startingBlockNumber;
                console.log(`Starting block number: ${startingBlockNumber}`);
            } else {
                currentBlockNumber = header.number.toNumber();
                console.log(`Current block number: ${currentBlockNumber}`);
            }

            if (currentBlockNumber >= startingBlockNumber + numberOfBlocks) {
                console.log(`Reached target block number: ${currentBlockNumber}`);
                resolve();
            }
        }).catch(reject);
    });
}


module.exports = { run };
