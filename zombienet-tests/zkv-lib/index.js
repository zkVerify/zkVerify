// Custom types and RPC calls
// This one defines the metadata for the return value of proofPath RPC call
zkvTypes = {
    MerkleProof: {
        root: 'H256',
        proof: 'Vec<H256>',
        number_of_leaves: 'u32',
        leaf_index: 'u32',
        leaf: 'H256',
    },
};

// This one defines the metadata for the arguments and return value of proofPath RPC call
zkvRpc = {
    aggregate: {
        statementPath: {
            description: 'Get the Merkle root and path of a aggregate statement',
            params: [
                {
                    name: 'at',
                    type: 'BlockHash',
                },
                {
                    name: 'domain_id',
                    type: 'u32'
                },
                {
                    name: 'aggregation_id',
                    type: 'u64'
                },
                {
                    name: 'statement',
                    type: 'H256'
                }
            ],
            type: 'MerkleProof'
        }
    }
};

BlockUntil = {
    InBlock: 'InBlock',

    Finalized: 'Finalized',
};

exports.BlockUntil = BlockUntil;

let api = null;

const BLOCK_TIME = 6000;  // block time in milliseconds
exports.BLOCK_TIME = BLOCK_TIME;

exports.init_api = async (zombie, nodeName, networkInfo) => {
    if (api === null) {
        const { wsUri } = networkInfo.nodesByName[nodeName];
        const provider = new zombie.WsProvider(wsUri);
        api = new zombie.ApiPromise({ provider, types: zkvTypes, rpc: zkvRpc });
        await api.isReady;
    }
    return api;
}

exports.submitProof = async (pallet, signer, ...verifierArgs) => {
    const validProofSubmission = (verifierArgs.length < 4) ? pallet.submitProof(...verifierArgs, null) : pallet.submitProof(...verifierArgs);
    return await submitExtrinsic(api, validProofSubmission, signer, BlockUntil.InBlock, (event) =>
        (event.method == "ProofVerified") ||
        (event.section == "aggregate" && event.method == "NewProof") ||
        (event.section == "aggregate" && event.method == "AggregationComplete")
    );
}

exports.registerDomain = async (signer, aggregation_size, queue_len, rules, destination, deliveryOwner) => {
    let extrinsic = api.tx.aggregate.registerDomain(aggregation_size, queue_len, rules, destination, deliveryOwner);
    return await submitExtrinsic(api, extrinsic, signer, BlockUntil.InBlock, (event) => event.section == "aggregate" && event.method == "NewDomain");
}

exports.sudoRegisterDomain = async (signer, aggregation_size, queue_len, rules, destination, deliveryOwner) => {
    let extrinsic = api.tx.sudo.sudo(api.tx.aggregate.registerDomain(aggregation_size, queue_len, rules, destination, deliveryOwner));
    return await submitExtrinsic(api, extrinsic, signer, BlockUntil.InBlock, (event) => event.section == "aggregate" && event.method == "NewDomain");
}

exports.unregisterDomain = async (signer, domain_id) => {
    let extrinsic = api.tx.aggregate.unregisterDomain(domain_id);
    return await submitExtrinsic(api, extrinsic, signer, BlockUntil.InBlock);
}

exports.holdDomain = async (signer, domain_id) => {
    let extrinsic = api.tx.aggregate.holdDomain(domain_id);
    return await submitExtrinsic(api, extrinsic, signer, BlockUntil.InBlock);
}

exports.aggregate = async (signer, domain_id, aggregation_id) => {
    let extrinsic = api.tx.aggregate.aggregate(domain_id, aggregation_id);
    return await submitExtrinsic(api, extrinsic, signer, BlockUntil.InBlock, (event) => event.section == "aggregate");
}

exports.waitForEvent = async (api, timeout, pallet, name) => {
    return await waitForEvent(api, timeout, pallet, name);
}

// Wait for the next attestaion id to be published
async function waitForEvent(api, timeout, pallet, name) {

    const retVal = await new Promise(async (resolve, reject) => {
        // Subscribe to system events via storage
        timeout = setTimeout(function () { unsubscribe(); reject("Timeout expired"); }, timeout);
        const unsubscribe = await api.query.system.events((events) => {
            console.log(`\nReceived ${events.length} events: `);

            // Loop through the Vec<EventRecord>
            events.forEach((record) => {
                // Extract the phase, event and the event types
                const { event, phase } = record;
                const types = event.typeDef;

                // Show what we are busy with
                console.log(`\t${event.section}: ${event.method}:: (phase = ${phase.toString()})`);

                if ((event.section == pallet) && (event.method == name)) {
                    clearTimeout(timeout);
                    unsubscribe();
                    resolve(event);
                }

                // Loop through each of the parameters, displaying the type and data
                event.data.forEach((data, index) => {
                    console.log(`\t\t\t${types[index].type}: ${data.toString()} `);
                });
            });
        });
    }).then(
        (ourBestEvent) => {
            console.log("A new event has been published")
            return ourBestEvent;
        },
        _error => {
            console.log("An error happened when waiting for the new event to be published.")
            return -1;
        }
    );

    return retVal;
}

exports.registerVk = async (pallet, signer, vk) => {
    return await submitExtrinsic(api, pallet.registerVk(vk), signer, BlockUntil.InBlock,
        (event) => event.section == "settlementUltraplonkPallet" && event.method == "VkRegistered"
    )
}

exports.submitExtrinsic = async (api, extrinsic, signer, blockUntil, filter) => {
    return await submitExtrinsic(api, extrinsic, signer, blockUntil, filter);
}

async function waitForEmptyMempool(api) {
    let pending = 0;
    do {
        await new Promise(r => setTimeout(r, BLOCK_TIME));
        pending = await api.rpc.author.pendingExtrinsics();
        console.log(`${pending.length} extrinsics pending in the mempool`);
    } while (pending.length > 0);
}

async function submitExtrinsic(api, extrinsic, signer, blockUntil, filter) {
    let transactionSuccessEvent = false;
    let done = false;
    let max_retries = 5;
    let hasBeenReady = false;
    if (filter === undefined) {
        console.log("No filtering");
        filter = (_event) => true;
    }

    let retVal = -1;
    while (!done && max_retries > 0) {
        retVal = await new Promise(async (resolve, reject) => {
            const unsub = await extrinsic.signAndSend(signer, ({ events: records = [], status }) => {
                let blockHash = null;
                if (status.isReady) {
                    hasBeenReady = true;
                }
                else if (status.isInBlock) {
                    blockHash = status.asInBlock;
                    console.log(`Transaction included at blockhash ${blockHash}`);
                    records.forEach(({ event: { method, section } }) => {
                        if (section == "system" && method == "ExtrinsicSuccess") {
                            transactionSuccessEvent = true;
                        }
                    });
                    if (blockUntil === BlockUntil.InBlock) {
                        done = true;
                    }
                }
                else if (status.isFinalized) {
                    console.log(`Transaction finalized at blockhash ${status.asFinalized}`);
                    if (blockUntil === BlockUntil.Finalized) {
                        done = true;
                    }
                }
                else if (status.isInvalid) {
                    console.log("Transaction marked as invalid");
                    done = true;
                    if (hasBeenReady) {
                        reject("retry");
                    }
                }
                else if (status.isError) {
                    done = true;
                    console.log("ERROR: Transaction status.isError");
                }
                if (done) {
                    unsub();
                    if (transactionSuccessEvent) {
                        resolve([blockHash, records]);
                    } else {
                        reject("ExtrinsicSuccess has not been seen");
                    }
                }
            }).catch(
                error => {
                    console.log(`Sending extrinsic failed with error: ${error}`);
                    if (error.code === 1014) { // priority too low error
                        reject("retry");
                    } else {
                        reject(error);
                    }
                }
            );
        }).then(
            ([blockHash, records]) => {
                console.log(`Transaction successfully processed [${blockHash}]: ${records}`);
                return {
                    block: blockHash,
                    events: records.map((record) => record.event).filter(filter)
                };
            },
            async function (error) {
                if (error !== "retry") {
                    console.log("Not retrying!");
                    return -1;
                }
                console.log("Transaction should be resubmitted, waiting for empty mempool...");
                max_retries -= 1;
                done = false;
                await waitForEmptyMempool(api);
            }
        );
    }

    return retVal;
}

exports.receivedEvents = (data) => {
    let events = Array.isArray(data) ? data : data.events;
    return Array.isArray(events) && events.length > 0;
}

exports.getBalance = async (user) => {
    return await getBalance(user);
}

async function getBalance(user) {
    return (await api.query.system.account(user.address))["data"]["free"]
}
