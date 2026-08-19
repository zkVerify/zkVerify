#!/usr/bin/env node

// Helpers to build the `pubs` blob for an AWS Nitro submission to the tee verifier.
//
// Usable both as a module (`require('./tee_nitro_policy.js')`) and from the command line
// (`./tee_nitro_policy.js --help`).
//
// The pubs are the canonical `NitroPolicy` encoding: the attestation values the
// submitter asserts the document must carry. Layout (variable size):
//
//   version ‖ pcr_bitmap ‖ flags ‖ pcr0..pcr15 ‖ [user_data_len ‖ user_data]
//
// `pcr0` (the enclave image measurement) is always checked; each other PCR only if
// its bitmap bit is set, and its slot must be all zeros otherwise. `user_data` is
// checked only if its flag bit is set, and the trailing length-prefixed bytes exist
// only in that case, so every policy has exactly one encoding.
//
// Mirrors tee-verifier/src/nitro/policy.rs.

const NITRO_POLICY_VERSION_V1 = 1;

const NITRO_PCR_SIZE = 48;
const NITRO_PCR_COUNT = 16;
const NITRO_MAX_USER_DATA_SIZE = 1024;

const VERSION_OFFSET = 0;
const BITMAP_OFFSET = VERSION_OFFSET + 1;
const FLAGS_OFFSET = BITMAP_OFFSET + 2;
const PCRS_OFFSET = FLAGS_OFFSET + 1;
const USER_DATA_LEN_OFFSET = PCRS_OFFSET + NITRO_PCR_COUNT * NITRO_PCR_SIZE;

const NITRO_POLICY_MIN_SIZE = USER_DATA_LEN_OFFSET;
const NITRO_POLICY_MAX_SIZE = NITRO_POLICY_MIN_SIZE + 2 + NITRO_MAX_USER_DATA_SIZE;

/// Only bit 0 of the flags byte is defined; any other bit makes the policy non-canonical.
const FLAG_USER_DATA = 1 << 0;

const PCR_NAMES = Array.from({ length: NITRO_PCR_COUNT }, (_, i) => `pcr${i}`);

/// What each PCR a Nitro enclave reports actually measures. Indices 4..15 are
/// reserved by AWS and reported as zero digests on current instances.
const PCR_DESCRIPTIONS = {
    pcr0: 'enclave image file (kernel, ramdisk, application) - always checked',
    pcr1: 'linux kernel and bootstrap ramdisk',
    pcr2: 'application ramdisk, excluding the boot ramdisk',
    pcr3: 'IAM role assigned to the parent instance',
    pcr4: 'instance ID of the parent instance',
    pcr8: 'signing certificate of the enclave image, when signed',
};

/// Coerce a hex string (with or without `0x`), Buffer, or Uint8Array to a Buffer.
function toBuffer(value, field) {
    if (Buffer.isBuffer(value)) {
        return value;
    }
    if (value instanceof Uint8Array) {
        return Buffer.from(value);
    }
    if (typeof value !== 'string') {
        throw new Error(`${field}: expected a hex string, Buffer, or Uint8Array`);
    }
    const hex = value.replace(/^0[xX]/, '');
    if (!/^[0-9a-fA-F]*$/.test(hex)) {
        throw new Error(`${field}: not a hex string`);
    }
    if (hex.length % 2 !== 0) {
        throw new Error(`${field}: hex string has an odd number of digits`);
    }
    return Buffer.from(hex, 'hex');
}

/// Coerce to a Buffer of exactly `size` bytes, naming the field on any mismatch.
function toBytes(value, size, field) {
    const buf = toBuffer(value, field);
    if (buf.length !== size) {
        throw new Error(`${field}: expected ${size} bytes, got ${buf.length}`);
    }
    return buf;
}

/// Build the canonical NitroPolicy encoding.
///
/// `fields.pcr0` (48 bytes) is mandatory - a policy that does not pin the enclave
/// image measurement is rejected on chain. `pcr1`..`pcr15` are optional. `userData`
/// is optional and distinguishes three cases: omitted means "not checked", while an
/// empty value pins user_data to empty (an absent user_data in the document counts
/// as empty). Values are hex strings, Buffers, or Uint8Arrays.
function buildNitroPolicy(fields) {
    const unknown = Object.keys(fields).filter(
        (k) => k !== 'userData' && !PCR_NAMES.includes(k),
    );
    if (unknown.length > 0) {
        throw new Error(`unknown policy field(s): ${unknown.join(', ')}`);
    }
    if (fields.pcr0 === undefined || fields.pcr0 === null) {
        throw new Error('pcr0 is mandatory: a policy that does not pin it is rejected');
    }

    const out = Buffer.alloc(NITRO_POLICY_MIN_SIZE); // zero-filled: unpinned slots stay zero
    out[VERSION_OFFSET] = NITRO_POLICY_VERSION_V1;

    let bitmap = 0;
    for (const [i, name] of PCR_NAMES.entries()) {
        const value = fields[name];
        if (value === undefined || value === null) {
            continue;
        }
        toBytes(value, NITRO_PCR_SIZE, name).copy(out, PCRS_OFFSET + i * NITRO_PCR_SIZE);
        bitmap |= 1 << i;
    }
    out.writeUInt16LE(bitmap, BITMAP_OFFSET);

    if (fields.userData === undefined || fields.userData === null) {
        return out;
    }
    const userData = toBuffer(fields.userData, 'userData');
    if (userData.length > NITRO_MAX_USER_DATA_SIZE) {
        throw new Error(
            `userData: ${userData.length} bytes exceeds the ${NITRO_MAX_USER_DATA_SIZE}-byte maximum`,
        );
    }
    out[FLAGS_OFFSET] |= FLAG_USER_DATA;
    const len = Buffer.alloc(2);
    len.writeUInt16LE(userData.length);
    return Buffer.concat([out, len, userData]);
}

/// Same as `buildNitroPolicy`, as a `0x`-prefixed hex string ready to pass as `pubs`
/// to `settlementTeePallet.submitProof`.
function buildNitroPolicyHex(fields) {
    return '0x' + buildNitroPolicy(fields).toString('hex');
}

/// Decode a pubs blob back into named fields, with unpinned PCRs as null. Applies the
/// same validation the on-chain parser does, so it doubles as a canonicality check.
function parseNitroPolicy(pubs) {
    const buf = toBuffer(pubs, 'pubs');
    if (buf.length < NITRO_POLICY_MIN_SIZE) {
        throw new Error(`pubs: expected at least ${NITRO_POLICY_MIN_SIZE} bytes, got ${buf.length}`);
    }
    if (buf[VERSION_OFFSET] !== NITRO_POLICY_VERSION_V1) {
        throw new Error(`unsupported policy version ${buf[VERSION_OFFSET]}`);
    }
    const bitmap = buf.readUInt16LE(BITMAP_OFFSET);
    if ((bitmap & 1) === 0) {
        throw new Error('pcr0 is not pinned (MissingPcr0)');
    }
    const flags = buf[FLAGS_OFFSET];
    if ((flags & ~FLAG_USER_DATA) !== 0) {
        throw new Error(`non-canonical policy: undefined flag bits set (0x${flags.toString(16)})`);
    }

    const out = { version: NITRO_POLICY_VERSION_V1, bitmap, flags };
    for (const [i, name] of PCR_NAMES.entries()) {
        const offset = PCRS_OFFSET + i * NITRO_PCR_SIZE;
        const slot = buf.subarray(offset, offset + NITRO_PCR_SIZE);
        if ((bitmap & (1 << i)) !== 0) {
            out[name] = '0x' + slot.toString('hex');
        } else if (slot.every((b) => b === 0)) {
            out[name] = null;
        } else {
            throw new Error(`non-canonical policy: ${name} is not pinned but is non-zero`);
        }
    }

    if ((flags & FLAG_USER_DATA) !== 0) {
        if (buf.length < USER_DATA_LEN_OFFSET + 2) {
            throw new Error('pubs: user_data is flagged but the length field is missing');
        }
        const len = buf.readUInt16LE(USER_DATA_LEN_OFFSET);
        if (len > NITRO_MAX_USER_DATA_SIZE) {
            throw new Error(`user_data length ${len} exceeds ${NITRO_MAX_USER_DATA_SIZE}`);
        }
        if (buf.length !== USER_DATA_LEN_OFFSET + 2 + len) {
            throw new Error(
                `pubs: expected ${USER_DATA_LEN_OFFSET + 2 + len} bytes for a ${len}-byte ` +
                `user_data, got ${buf.length}`,
            );
        }
        out.userData = '0x' + buf.subarray(USER_DATA_LEN_OFFSET + 2).toString('hex');
    } else {
        if (buf.length !== NITRO_POLICY_MIN_SIZE) {
            throw new Error(
                `non-canonical policy: user_data is not pinned but the encoding is ` +
                `${buf.length} bytes instead of ${NITRO_POLICY_MIN_SIZE}`,
            );
        }
        out.userData = null;
    }
    return out;
}

// =============================================================================
// Attestation document (COSE_Sign1 / CBOR)
// =============================================================================

/// Minimal CBOR reader covering what Nitro attestation documents use: definite and
/// indefinite lengths for strings, arrays and maps. Byte strings come back as Buffers,
/// maps as Maps (keys may be integers or text).
const BREAK = Symbol('cbor-break');

function cborRead(buf, pos) {
    if (pos >= buf.length) {
        throw new Error('CBOR: unexpected end of input');
    }
    const initial = buf[pos++];
    const major = initial >> 5;
    const info = initial & 0x1f;

    if (initial === 0xff) {
        return [BREAK, pos];
    }

    let value = info;
    if (info === 24) {
        value = buf[pos++];
    } else if (info === 25) {
        value = buf.readUInt16BE(pos);
        pos += 2;
    } else if (info === 26) {
        value = buf.readUInt32BE(pos);
        pos += 4;
    } else if (info === 27) {
        value = Number(buf.readBigUInt64BE(pos));
        pos += 8;
    } else if (info >= 28 && info <= 30) {
        throw new Error(`CBOR: reserved additional info ${info}`);
    }

    // Indefinite length: items follow until a break byte. Strings arrive as a run of
    // definite-length chunks to concatenate; arrays and maps as their elements.
    const indefinite = info === 31;

    switch (major) {
        case 0:
            return [value, pos];
        case 1:
            return [-1 - value, pos];
        case 2:
        case 3: {
            let bytes;
            if (indefinite) {
                const chunks = [];
                for (;;) {
                    const [chunk, next] = cborRead(buf, pos);
                    pos = next;
                    if (chunk === BREAK) {
                        break;
                    }
                    if (Buffer.isBuffer(chunk)) {
                        chunks.push(chunk);
                    } else if (typeof chunk === 'string') {
                        chunks.push(Buffer.from(chunk, 'utf8'));
                    } else {
                        throw new Error('CBOR: indefinite string chunk is not a string');
                    }
                }
                bytes = Buffer.concat(chunks);
            } else {
                bytes = buf.subarray(pos, pos + value);
                if (bytes.length !== value) {
                    throw new Error('CBOR: truncated string');
                }
                bytes = Buffer.from(bytes);
                pos += value;
            }
            return [major === 2 ? bytes : bytes.toString('utf8'), pos];
        }
        case 4: {
            const arr = [];
            for (let i = 0; indefinite || i < value; i++) {
                const [item, next] = cborRead(buf, pos);
                pos = next;
                if (item === BREAK) {
                    if (!indefinite) {
                        throw new Error('CBOR: unexpected break in a definite-length array');
                    }
                    break;
                }
                arr.push(item);
            }
            return [arr, pos];
        }
        case 5: {
            const map = new Map();
            for (let i = 0; indefinite || i < value; i++) {
                const [k, afterKey] = cborRead(buf, pos);
                pos = afterKey;
                if (k === BREAK) {
                    if (!indefinite) {
                        throw new Error('CBOR: unexpected break in a definite-length map');
                    }
                    break;
                }
                const [v, afterValue] = cborRead(buf, pos);
                map.set(k, v);
                pos = afterValue;
            }
            return [map, pos];
        }
        case 6:
            // Tag: Nitro documents may carry the COSE_Sign1 tag (18); the content is
            // what matters.
            return cborRead(buf, pos);
        default:
            if (info === 20) return [false, pos];
            if (info === 21) return [true, pos];
            if (info === 22) return [null, pos];
            throw new Error(`CBOR: unsupported simple value ${info}`);
    }
}

/// Extract the payload map of a Nitro attestation document: a COSE_Sign1 array of
/// [protected, unprotected, payload, signature] whose payload is itself CBOR.
function attestationPayload(doc) {
    const buf = toBuffer(doc, 'attestation');
    const [outer] = cborRead(buf, 0);
    if (!Array.isArray(outer) || outer.length !== 4) {
        throw new Error('attestation: not a COSE_Sign1 array of 4 elements');
    }
    if (!Buffer.isBuffer(outer[2])) {
        throw new Error('attestation: COSE_Sign1 payload is not a byte string');
    }
    const [payload] = cborRead(outer[2], 0);
    if (!(payload instanceof Map)) {
        throw new Error('attestation: payload is not a CBOR map');
    }
    return payload;
}

/// Read the PCRs and user_data out of an attestation document and build the fields for
/// `buildNitroPolicy`, pinning `pin`.
///
/// `pin` is 'all' (every reported PCR plus user_data), 'none' (pcr0 only, the minimum
/// the chain accepts), or a list of field names such as ['pcr0', 'pcr8', 'userData'].
/// pcr0 is always included because a policy without it is rejected.
///
/// Reading the expected values back out of the document is for tests and inspection;
/// in production the measurements should be ones you decided on out of band.
function nitroPolicyFromAttestation(doc, pin = 'all') {
    const payload = attestationPayload(doc);
    const pcrs = payload.get('pcrs');
    if (!(pcrs instanceof Map)) {
        throw new Error('attestation: payload has no pcrs map');
    }
    const reported = new Map();
    for (const [index, digest] of pcrs) {
        if (typeof index === 'number' && Buffer.isBuffer(digest)) {
            reported.set(index, digest);
        }
    }
    const userData = payload.get('user_data');

    const names = pin === 'all'
        ? [...PCR_NAMES, 'userData']
        : pin === 'none'
          ? ['pcr0']
          : pin;
    const unknown = names.filter((n) => n !== 'userData' && !PCR_NAMES.includes(n));
    if (unknown.length > 0) {
        throw new Error(`cannot pin unknown field(s): ${unknown.join(', ')}`);
    }

    const fields = {};
    for (const name of new Set(['pcr0', ...names])) {
        if (name === 'userData') {
            // An absent user_data in the document counts as empty bytes.
            fields.userData = Buffer.isBuffer(userData) ? userData : Buffer.alloc(0);
            continue;
        }
        const index = PCR_NAMES.indexOf(name);
        const digest = reported.get(index);
        if (digest === undefined) {
            throw new Error(`attestation does not report ${name}, so it cannot be pinned`);
        }
        if (digest.length !== NITRO_PCR_SIZE) {
            throw new Error(
                `attestation reports ${name} as ${digest.length} bytes, expected ${NITRO_PCR_SIZE}`,
            );
        }
        fields[name] = digest;
    }
    return fields;
}

// =============================================================================
// Field inventory
// =============================================================================

/// Every field of the pubs encoding, in byte order - the exhaustive set a policy can
/// carry. `bit` is the PCR bitmap mask; `flag` the flags-byte mask; `conditional`
/// marks the tail that exists only when user_data is pinned.
const POLICY_FIELDS = [
    {
        name: 'version',
        offset: VERSION_OFFSET,
        size: 1,
        bit: null,
        flag: null,
        conditional: false,
        kind: 'header',
        description: `encoding version, must be ${NITRO_POLICY_VERSION_V1}`,
    },
    {
        name: 'pcr_bitmap',
        offset: BITMAP_OFFSET,
        size: 2,
        bit: null,
        flag: null,
        conditional: false,
        kind: 'header',
        description: 'little-endian u16, which PCRs are pinned; derived, bit 0 required',
    },
    {
        name: 'flags',
        offset: FLAGS_OFFSET,
        size: 1,
        bit: null,
        flag: null,
        conditional: false,
        kind: 'header',
        description: 'only bit 0 (user_data pinned) is defined; any other bit is rejected',
    },
    ...PCR_NAMES.map((name, i) => ({
        name,
        offset: PCRS_OFFSET + i * NITRO_PCR_SIZE,
        size: NITRO_PCR_SIZE,
        bit: 1 << i,
        flag: null,
        conditional: false,
        kind: i === 0 ? 'always checked' : 'optional',
        description: PCR_DESCRIPTIONS[name] || 'reserved by AWS; reported as a zero digest',
    })),
    {
        name: 'user_data_len',
        offset: USER_DATA_LEN_OFFSET,
        size: 2,
        bit: null,
        flag: FLAG_USER_DATA,
        conditional: true,
        kind: 'optional',
        description: 'little-endian u16, present only when user_data is pinned',
    },
    {
        name: 'user_data',
        offset: USER_DATA_LEN_OFFSET + 2,
        size: `0..${NITRO_MAX_USER_DATA_SIZE}`,
        bit: null,
        flag: FLAG_USER_DATA,
        conditional: true,
        kind: 'optional',
        description: 'bytes the enclave supplied; absent in the document counts as empty',
    },
];

/// Attestation-document fields with no counterpart in the policy encoding: values a
/// submitter cannot pin through the pubs.
const ATTESTATION_ONLY_FIELDS = [
    { name: 'module_id', enforcement: 'parsed only; never compared' },
    {
        name: 'timestamp',
        enforcement:
            "parsed only; never compared - the document's own age is not checked, freshness " +
            "comes only from the signing certificate's short validity window",
    },
    {
        name: 'digest',
        enforcement: 'must be exactly "SHA384", enforced by verify(); not policy-checked',
    },
    {
        name: 'certificate',
        enforcement: 'the signing certificate - verified against cabundle, the AWS root and the CRL',
    },
    { name: 'cabundle', enforcement: 'the certificate chain - verified, not policy-checked' },
    { name: 'public_key', enforcement: 'parsed only; never compared' },
    { name: 'nonce', enforcement: 'parsed only; never compared - bind freshness via user_data instead' },
];

// =============================================================================
// CLI
// =============================================================================

const USAGE = `Build or inspect the pubs blob (canonical NitroPolicy) for an AWS Nitro
submission to the tee verifier.

Usage:
  tee_nitro_policy.js from-attestation <doc> [--pin <fields>] [--json]
  tee_nitro_policy.js build --pcr0 <v> [--pcr<n> <v> ...] [--user-data <v>] [--json]
  tee_nitro_policy.js parse <pubs> [--json]
  tee_nitro_policy.js fields [--json]

Commands:
  from-attestation  Read the PCRs and user_data out of an attestation document and pin
                    the requested ones. For tests and inspection: in production the
                    expected measurements should be values decided out of band, not
                    copied from the document being verified.
  build             Assemble a policy from values given on the command line.
  parse             Decode a pubs blob into named fields, applying the same validation
                    the on-chain parser does. Unpinned fields print as null.
  fields            List every field the encoding can carry - offset, size, bitmap bit
                    and meaning - plus the document fields that no policy can pin.

Options:
  --pin <fields>  from-attestation only: 'all' (default: every PCR plus user_data),
                  'none' (pcr0 only), or a comma-separated list such as
                  'pcr0,pcr8,userData'. pcr0 is always included.
  --json          Print the decoded fields as JSON instead of just the pubs hex.
  -h, --help      Show this help.

Fields for 'build': pcr0 (${NITRO_PCR_SIZE} bytes) is mandatory - a policy that does not pin the
enclave image measurement is rejected on chain. pcr1..pcr${NITRO_PCR_COUNT - 1} are optional, ${NITRO_PCR_SIZE} bytes each.
--user-data pins up to ${NITRO_MAX_USER_DATA_SIZE} bytes; passing '' pins it to empty, omitting it leaves
user_data unchecked. Encoding is ${NITRO_POLICY_MIN_SIZE} bytes without user_data, up to ${NITRO_POLICY_MAX_SIZE} with it.

<doc>, <pubs> and every field value may be a hex string (with or without 0x), a path
to a file holding either hex text or raw bytes, or '-' to read stdin.

Examples:
  tee_nitro_policy.js from-attestation doc.bin > pubs.hex
  tee_nitro_policy.js from-attestation doc.bin --pin pcr0,pcr1,pcr2 --json
  tee_nitro_policy.js build --pcr0 0x22f9... --pcr8 0x0000... --user-data 0xdeadbeef
  tee_nitro_policy.js parse pubs.hex --json
`;

const HEX_TEXT = /^(0[xX])?[0-9a-fA-F\s]+$/;

/// Resolve a CLI argument to bytes: '-' reads stdin, an existing path reads the file
/// (hex text or raw bytes), anything else is treated as a hex literal.
function readValue(arg, field) {
    const fs = require('fs');
    if (arg === '-') {
        return fromHexOrRaw(fs.readFileSync(0), field);
    }
    if (arg !== '' && fs.existsSync(arg) && fs.statSync(arg).isFile()) {
        return fromHexOrRaw(fs.readFileSync(arg), field);
    }
    return toBuffer(arg, field);
}

/// Interpret file/stdin content as hex text when it looks like hex, else as raw bytes.
function fromHexOrRaw(buf, field) {
    const text = buf.toString('utf8');
    if (buf.length > 0 && HEX_TEXT.test(text)) {
        return toBuffer(text.replace(/\s+/g, ''), field);
    }
    return buf;
}

/// Minimal `--flag value` parser. Returns { positional, flags }; boolean flags are
/// listed in `booleans` and take no value.
function parseArgv(argv, booleans) {
    const positional = [];
    const flags = {};
    for (let i = 0; i < argv.length; i++) {
        const arg = argv[i];
        if (!arg.startsWith('--')) {
            positional.push(arg);
            continue;
        }
        const name = arg.slice(2);
        if (booleans.includes(name)) {
            flags[name] = true;
        } else {
            if (i + 1 >= argv.length) {
                throw new Error(`--${name} requires a value`);
            }
            flags[name] = argv[++i];
        }
    }
    return { positional, flags };
}

/// Map a `--pcr3` / `--user-data` flag name to its field name, or null if unknown.
function fieldForFlag(name) {
    if (name === 'user-data' || name === 'userData') {
        return 'userData';
    }
    return PCR_NAMES.includes(name) ? name : null;
}

function describe(pubs) {
    const parsed = parseNitroPolicy(pubs);
    return JSON.stringify(
        {
            pubs: '0x' + pubs.toString('hex'),
            size: pubs.length,
            version: parsed.version,
            pcrBitmap: '0x' + parsed.bitmap.toString(16).padStart(4, '0'),
            flags: '0x' + parsed.flags.toString(16).padStart(2, '0'),
            fields: Object.fromEntries(
                [...PCR_NAMES, 'userData'].map((f) => [f, parsed[f]]),
            ),
        },
        null,
        2,
    );
}

function printFields(json) {
    if (json) {
        process.stdout.write(
            JSON.stringify(
                {
                    minSize: NITRO_POLICY_MIN_SIZE,
                    maxSize: NITRO_POLICY_MAX_SIZE,
                    version: NITRO_POLICY_VERSION_V1,
                    fields: POLICY_FIELDS,
                    attestationOnlyFields: ATTESTATION_ONLY_FIELDS,
                },
                null,
                2,
            ) + '\n',
        );
        return;
    }
    const p = (line) => process.stdout.write(line + '\n');
    p(`pubs encoding: ${NITRO_POLICY_MIN_SIZE} bytes without user_data, ` +
      `up to ${NITRO_POLICY_MAX_SIZE} with it, version ${NITRO_POLICY_VERSION_V1}`);
    p('');
    p('  off  size    bit     field          kind            meaning');
    for (const f of POLICY_FIELDS) {
        const mark = f.bit !== null
            ? '0x' + f.bit.toString(16).padStart(4, '0')
            : f.flag !== null
              ? 'flag ' + f.flag
              : '-';
        p(`  ${String(f.offset).padStart(3)}  ${String(f.size).padStart(6)}  ${mark.padEnd(6)}  ` +
          `${f.name.padEnd(14)} ${f.kind.padEnd(15)} ${f.description}`);
    }
    p('');
    p('Attestation document fields with no policy slot (cannot be pinned through pubs):');
    for (const f of ATTESTATION_ONLY_FIELDS) {
        p(`  ${f.name.padEnd(14)} ${f.enforcement}`);
    }
}

function main(argv) {
    if (argv.includes('-h') || argv.includes('--help')) {
        process.stdout.write(USAGE);
        return 0;
    }
    if (argv.length === 0) {
        process.stderr.write(USAGE);
        return 2;
    }

    const command = argv[0];
    const { positional, flags } = parseArgv(argv.slice(1), ['json']);

    if (command === 'fields') {
        printFields(flags.json);
        return 0;
    }

    let pubs;
    switch (command) {
        case 'from-attestation': {
            if (positional.length !== 1) {
                throw new Error('from-attestation takes exactly one <doc> argument');
            }
            const spec = flags.pin === undefined ? 'all' : flags.pin;
            const pin = spec === 'all' || spec === 'none'
                ? spec
                : spec.split(',').map((f) => f.trim()).filter((f) => f !== '');
            pubs = buildNitroPolicy(
                nitroPolicyFromAttestation(readValue(positional[0], 'attestation'), pin),
            );
            break;
        }
        case 'build': {
            if (positional.length !== 0) {
                throw new Error(`unexpected argument: ${positional[0]}`);
            }
            const fields = {};
            for (const [name, value] of Object.entries(flags)) {
                if (name === 'json') {
                    continue;
                }
                const field = fieldForFlag(name);
                if (field === null) {
                    throw new Error(`unknown policy field: --${name}`);
                }
                fields[field] = readValue(value, field);
            }
            pubs = buildNitroPolicy(fields);
            break;
        }
        case 'parse': {
            if (positional.length !== 1) {
                throw new Error('parse takes exactly one <pubs> argument');
            }
            process.stdout.write(describe(readValue(positional[0], 'pubs')) + '\n');
            return 0;
        }
        default:
            throw new Error(`unknown command: ${command} (try --help)`);
    }

    process.stdout.write((flags.json ? describe(pubs) : '0x' + pubs.toString('hex')) + '\n');
    return 0;
}

if (require.main === module) {
    try {
        process.exitCode = main(process.argv.slice(2));
    } catch (e) {
        process.stderr.write(`error: ${e.message}\n`);
        process.exitCode = 1;
    }
}

exports.NITRO_POLICY_VERSION_V1 = NITRO_POLICY_VERSION_V1;
exports.NITRO_POLICY_MIN_SIZE = NITRO_POLICY_MIN_SIZE;
exports.NITRO_POLICY_MAX_SIZE = NITRO_POLICY_MAX_SIZE;
exports.NITRO_PCR_SIZE = NITRO_PCR_SIZE;
exports.NITRO_PCR_COUNT = NITRO_PCR_COUNT;
exports.NITRO_MAX_USER_DATA_SIZE = NITRO_MAX_USER_DATA_SIZE;
exports.buildNitroPolicy = buildNitroPolicy;
exports.buildNitroPolicyHex = buildNitroPolicyHex;
exports.nitroPolicyFromAttestation = nitroPolicyFromAttestation;
exports.attestationPayload = attestationPayload;
exports.parseNitroPolicy = parseNitroPolicy;
exports.POLICY_FIELDS = POLICY_FIELDS;
exports.ATTESTATION_ONLY_FIELDS = ATTESTATION_ONLY_FIELDS;
