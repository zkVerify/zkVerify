#! /usr/bin/env node
/*
  No-sudo E2E feasibility run:
  - register domains (Destination::None)
  - submit real proofs to fill queue
  - batch aggregate all ready aggregations
  - report block + Published size
*/

const {ApiPromise, WsProvider, Keyring} = require('@polkadot/api');
const {VK, PROOF, PUBS} = require('/home/mdamico/devel/zkVerify/zombienet-tests/js_scripts/groth16_data.js');

const WS = process.env.WS || 'ws://127.0.0.1:9944';
const DOMAINS = Number.parseInt(process.env.DOMAINS || '2', 10);
const QUEUE_SIZE = Number.parseInt(process.env.QUEUE_SIZE || '1', 10); // completed aggregations per domain
const AGG_SIZE_OVERRIDE = process.env.AGG_SIZE ? Number.parseInt(process.env.AGG_SIZE, 10) : null;

function sleep(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

async function submitAndWait(api, signer, tx, label) {
    return new Promise(async (resolve, reject) => {
        let includedHash = null;
        let unsub;
        try {
            unsub = await tx.signAndSend(signer, ({status, dispatchError, events}) => {
                if (status.isInBlock) {
                    includedHash = status.asInBlock.toHex();
                }
                if (dispatchError) {
                    let msg = dispatchError.toString();
                    if (dispatchError.isModule) {
                        const meta = api.registry.findMetaError(dispatchError.asModule);
                        msg = `${meta.section}.${meta.name}`;
                    }
                    unsub();
                    reject(new Error(`${label} failed: ${msg}`));
                }
                if (status.isFinalized) {
                    const success = events.some(
                        ({event}) => event.section === 'system' && event.method === 'ExtrinsicSuccess'
                    );
                    unsub();
                    if (!success) {
                        reject(new Error(`${label} finalized without ExtrinsicSuccess`));
                        return;
                    }
                    resolve({blockHash: includedHash || status.asFinalized.toHex(), events});
                }
            });
        } catch (err) {
            reject(err);
        }
    });
}

async function connectWithRetry(ws, retries = 60, delayMs = 1000) {
    let lastErr;
    for (let i = 0; i < retries; i++) {
        try {
            const api = await ApiPromise.create({provider: new WsProvider(ws)});
            await api.isReady;
            return api;
        } catch (err) {
            lastErr = err;
            await sleep(delayMs);
        }
    }
    throw lastErr || new Error('Unable to connect');
}

function findSubmitProofSection(api) {
    const sections = Object.keys(api.tx).filter((s) => api.tx[s].submitProof);
    const preferred = sections.find((s) => s.toLowerCase().includes('groth16'));
    return preferred || sections[0];
}

async function getDomainInfo(api, domainId) {
    const domainOpt = await api.query.aggregate.domains(domainId);
    if (domainOpt.isNone) {
        return null;
    }
    const domain = domainOpt.unwrap();
    const json = domain.toJSON();
    const shouldPublish = json.shouldPublish || {};
    const shouldPublishIds = Object.keys(shouldPublish).map((k) => Number.parseInt(k, 10));
    const nextId = Number.parseInt(json.next.id, 10);
    const nextStatementsLen = json.next.statements.length;
    return {
        shouldPublishIds,
        shouldPublishLen: shouldPublishIds.length,
        nextId,
        nextStatementsLen,
    };
}

async function main() {
    const api = await connectWithRetry(WS);
    const keyring = new Keyring({type: 'sr25519'});
    const alice = keyring.addFromUri('//Alice');

    const aggSizeConst = api.consts.aggregate.aggregationSize || api.consts.aggregate.aggregateMaxSize;
    const aggSize = AGG_SIZE_OVERRIDE || aggSizeConst.toNumber();

    const delivery = {destination: {None: null}, fee: 0, ownerTip: 0};

    const submitSection = findSubmitProofSection(api);
    if (!submitSection) {
        throw new Error('No submitProof section found in api.tx');
    }
    console.log(`INFO submitProof pallet: ${submitSection}`);

    const submitProof = (domainId) =>
        api.tx[submitSection].submitProof({Vk: VK}, PROOF, PUBS, domainId);

    const domainIds = [];
    for (let i = 0; i < DOMAINS; i++) {
        const tx = api.tx.aggregate.registerDomain(
            aggSize,
            QUEUE_SIZE,
            'Untrusted',
            'Untrusted',
            delivery,
            null
        );
        const res = await submitAndWait(api, alice, tx, `registerDomain(${i})`);
        const newDomainEvt = res.events.find(
            ({event}) => event.section === 'aggregate' && event.method === 'NewDomain'
        );
        if (!newDomainEvt) {
            throw new Error('NewDomain event not found');
        }
        const domainId = Number.parseInt(newDomainEvt.event.data[0].toString(), 10);
        domainIds.push(domainId);
        console.log(`INFO registered domainId=${domainId}`);
    }

    const blockWeights = api.consts.system.blockWeights;
    const maxTotalOpt = blockWeights.perClass.normal.maxTotal;
    const maxTotal = maxTotalOpt.isSome ? maxTotalOpt.unwrap() : blockWeights.maxBlock;
    const maxRefTime = maxTotal.refTime.toBn();
    const maxProofSize = maxTotal.proofSize.toBn();
    const maxExtrinsicOpt = blockWeights.perClass.normal.maxExtrinsic;
    const maxExtrinsic = maxExtrinsicOpt && maxExtrinsicOpt.isSome ? maxExtrinsicOpt.unwrap() : maxTotal;
    const maxExtrinsicRefTime = maxExtrinsic.refTime.toBn();
    const maxExtrinsicProofSize = maxExtrinsic.proofSize.toBn();
    const blockLength = api.consts.system.blockLength;
    const maxLen = blockLength.max.normal.toNumber();

    console.info(`maxExtrinsicRefTime = ${maxExtrinsicRefTime}`);
    console.info(`maxExtrinsicProofSize = ${maxExtrinsicProofSize}`);
    console.info(`maxLen = ${maxLen}`);

    const tx1 = api.tx.utility.batchAll([submitProof(domainIds[0])]);
    const unsignedLen1 = tx1.toU8a().length;
    await tx1.signAsync(alice);
    const signedLen1 = tx1.toU8a().length;
    const sigOverhead = signedLen1 - unsignedLen1;

    async function fitsProofBatch(m) {
        if (m === 0) return {ok: true, weight: null, len: 0};
        const calls = Array.from({length: m}, () => submitProof(domainIds[0]));
        const tx = api.tx.utility.batchAll(calls);
        const info = await tx.paymentInfo(alice);
        const weight = info.weight;
        const okWeight =
            weight.refTime.toBn().lte(maxRefTime) && weight.proofSize.toBn().lte(maxProofSize);
        const len = tx.toU8a().length + sigOverhead;
        const okLen = len <= maxLen;
        return {ok: okWeight && okLen, weight, len};
    }

    let lo = 0;
    let hi = 1;
    while (hi <= 128) {
        const {ok} = await fitsProofBatch(hi);
        if (!ok) break;
        lo = hi;
        hi *= 2;
    }
    while (lo + 1 < hi) {
        const mid = Math.floor((lo + hi) / 2);
        const {ok} = await fitsProofBatch(mid);
        if (ok) lo = mid;
        else hi = mid;
    }
    const maxProofsPerBatch = Math.max(1, lo);
    console.log(`INFO max proofs per batch = ${maxProofsPerBatch}`);

    const targetPerDomain = QUEUE_SIZE * aggSize + 1;
    console.log(`INFO target proofs per domain = ${targetPerDomain}`);

    for (const domainId of domainIds) {
        let info = await getDomainInfo(api, domainId);
        while (info.shouldPublishLen < QUEUE_SIZE || info.nextStatementsLen === 0) {
            const remaining = targetPerDomain - (info.shouldPublishLen * aggSize + info.nextStatementsLen);
            const batchSize = Math.max(1, Math.min(maxProofsPerBatch, remaining));
            const calls = Array.from({length: batchSize}, () => submitProof(domainId));
            const batch = api.tx.utility.batchAll(calls);
            await submitAndWait(api, alice, batch, `submitProof batch domain=${domainId}`);
            info = await getDomainInfo(api, domainId);
            console.log(
                `INFO domain=${domainId} should_publish=${info.shouldPublishLen} next_statements=${info.nextStatementsLen}`
            );
        }
    }

    const aggregateCalls = [];
    for (const domainId of domainIds) {
        const info = await getDomainInfo(api, domainId);
        for (const aggId of info.shouldPublishIds) {
            aggregateCalls.push(api.tx.aggregate.aggregate(domainId, aggId));
        }
        aggregateCalls.push(api.tx.aggregate.aggregate(domainId, info.nextId));
    }

    async function fitsAggregateBatch(m) {
        if (m === 0) return {ok: true, weight: null, len: 0};
        const calls = Array.from({length: m}, () => api.tx.aggregate.aggregate(domainIds[0] || 0, 0));
        const tx = api.tx.utility.batchAll(calls);
        const info = await tx.paymentInfo(alice);
        const weight = info.weight;
        const okWeight =
            weight.refTime.toBn().lte(maxExtrinsicRefTime) &&
            weight.proofSize.toBn().lte(maxExtrinsicProofSize);
        const len = tx.toU8a().length + sigOverhead;
        const okLen = len <= maxLen;
        return {ok: okWeight && okLen, weight, len};
    }

    let alo = 0;
    let ahi = 1;
    while (ahi <= 512) {
        const {ok} = await fitsAggregateBatch(ahi);
        if (!ok) break;
        alo = ahi;
        ahi *= 2;
    }
    while (alo + 1 < ahi) {
        const mid = Math.floor((alo + ahi) / 2);
        const {ok, weight} = await fitsAggregateBatch(mid);
        console.log(`weight for aggregate batch size ${mid}: refTime=${weight.refTime.toBn()}}`);
        if (ok) alo = mid;
        else ahi = mid;
    }
    const maxAggCalls = Math.max(1, alo);
    console.log(`INFO max aggregate calls per batch = ${maxAggCalls}`);

    const cappedCalls =
        aggregateCalls.length > maxAggCalls ? aggregateCalls.slice(0, maxAggCalls) : aggregateCalls;
    if (cappedCalls.length !== aggregateCalls.length) {
        console.log(
            `INFO capping aggregate calls from ${aggregateCalls.length} to ${cappedCalls.length}`
        );
    }

    const batchAgg = api.tx.utility.batchAll(cappedCalls);
    const resAgg = await submitAndWait(api, alice, batchAgg, 'batchAll(aggregate)');
    const blockHash = resAgg.blockHash;
    const header = await api.rpc.chain.getHeader(blockHash);
    const blockNumber = header.number.toNumber();

    const published = await api.query.aggregate.published.at(blockHash);
    const publishedLen = published.length;
    const publishedBytes = published.toU8a().length;

    console.log(
        `RESULT DOMAINS=${DOMAINS} QUEUE_SIZE=${QUEUE_SIZE} AGG_SIZE=${aggSize} BATCH_AGG_CALLS=${aggregateCalls.length} BLOCK_NUM=${blockNumber} BLOCK_HASH=${blockHash} PUBLISHED_LEN=${publishedLen} PUBLISHED_BYTES=${publishedBytes}`
    );

    await api.disconnect();
}

main().catch((err) => {
    console.error(err);
    process.exit(1);
});
