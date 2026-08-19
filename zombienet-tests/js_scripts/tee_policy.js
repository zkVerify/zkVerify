#!/usr/bin/env node

// Helpers to build the `pubs` blob for an Intel TDX submission to the tee verifier.
//
// Usable both as a module (`require('./tee_policy.js')`) and from the command line
// (`./tee_policy.js --help`).
//
// The pubs are the canonical `TdReportPolicy` encoding: the TD report values the
// submitter asserts the quote must carry. Layout (458 bytes, fixed):
//
//   version ‖ bitmap ‖ xfam ‖ mrtd ‖ mrconfigid ‖ mrowner ‖ mrownerconfig ‖ rtmr0..3 ‖ reportdata
//
// `mrtd` and `reportdata` are always checked. Every other field is checked only if
// its bitmap bit is set, and must be all zeros otherwise, so each policy has exactly
// one encoding (the statement hash covers keccak256(pubs)).
//
// Mirrors tee-verifier/src/intel/{policy,constants}.rs.

const POLICY_VERSION_V1 = 1;

// Field sizes, in encoding order.
const SIZES = {
    version: 1,
    bitmap: 1,
    xfam: 8,
    mrtd: 48,
    mrconfigid: 48,
    mrowner: 48,
    mrownerconfig: 48,
    rtmr0: 48,
    rtmr1: 48,
    rtmr2: 48,
    rtmr3: 48,
    reportdata: 64,
};

// Offsets derived from the sizes, so the two can never drift apart.
const OFFSETS = {};
let cursor = 0;
for (const [field, size] of Object.entries(SIZES)) {
    OFFSETS[field] = cursor;
    cursor += size;
}
const TD_REPORT_POLICY_SIZE = cursor; // 458

// Presence bitmap bits, one per optional field. `mrtd` and `reportdata` have no bit:
// they are always checked.
const BITS = {
    xfam: 1 << 0,
    mrconfigid: 1 << 1,
    mrowner: 1 << 2,
    mrownerconfig: 1 << 3,
    rtmr0: 1 << 4,
    rtmr1: 1 << 5,
    rtmr2: 1 << 6,
    rtmr3: 1 << 7,
};

const MANDATORY = ['mrtd', 'reportdata'];
const OPTIONAL = Object.keys(BITS);

// Quote layout, only as much as is needed to read the TD report body out of a quote.
const QUOTE_HEADER_SIZE = 48;
// TD report body fields in order, so the offsets below are derived, not transcribed.
const BODY_LAYOUT = [
    ['tee_tcb_svn', 16],
    ['mrseam', 48],
    ['mrsignerseam', 48],
    ['seamattributes', 8],
    ['tdattributes', 8],
    ['xfam', 8],
    ['mrtd', 48],
    ['mrconfigid', 48],
    ['mrowner', 48],
    ['mrownerconfig', 48],
    ['rtmr0', 48],
    ['rtmr1', 48],
    ['rtmr2', 48],
    ['rtmr3', 48],
    ['reportdata', 64],
];
const BODY_OFFSETS = {};
let bodyCursor = 0;
for (const [field, size] of BODY_LAYOUT) {
    BODY_OFFSETS[field] = bodyCursor;
    bodyCursor += size;
}
const QUOTE_BODY_SIZE = bodyCursor; // 584

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

/// Build the canonical TdReportPolicy encoding.
///
/// `fields.mrtd` (48 bytes) and `fields.reportdata` (64 bytes) are mandatory. Any of
/// `xfam` (8 bytes), `mrconfigid`, `mrowner`, `mrownerconfig`, `rtmr0`..`rtmr3` (48
/// bytes each) may be supplied to pin it; omit (or pass null/undefined) to leave it
/// unchecked. Values are hex strings, Buffers, or Uint8Arrays.
///
/// Returns a Buffer; use `buildTdReportPolicyHex` for a `0x`-prefixed string.
function buildTdReportPolicy(fields) {
    const unknown = Object.keys(fields).filter(
        (k) => !MANDATORY.includes(k) && !OPTIONAL.includes(k),
    );
    if (unknown.length > 0) {
        throw new Error(`unknown policy field(s): ${unknown.join(', ')}`);
    }

    const out = Buffer.alloc(TD_REPORT_POLICY_SIZE); // zero-filled: unpinned fields stay zero
    out[OFFSETS.version] = POLICY_VERSION_V1;

    for (const field of MANDATORY) {
        if (fields[field] === undefined || fields[field] === null) {
            throw new Error(`${field} is mandatory: it is always checked against the quote`);
        }
        toBytes(fields[field], SIZES[field], field).copy(out, OFFSETS[field]);
    }

    let bitmap = 0;
    for (const field of OPTIONAL) {
        const value = fields[field];
        if (value === undefined || value === null) {
            continue;
        }
        toBytes(value, SIZES[field], field).copy(out, OFFSETS[field]);
        bitmap |= BITS[field];
    }
    out[OFFSETS.bitmap] = bitmap;

    return out;
}

/// Same as `buildTdReportPolicy`, as a `0x`-prefixed hex string ready to pass as
/// `pubs` to `settlementTeePallet.submitProof`.
function buildTdReportPolicyHex(fields) {
    return '0x' + buildTdReportPolicy(fields).toString('hex');
}

/// Read the TD report values out of a quote and build a policy pinning `pin`.
///
/// `pin` is a list of optional field names, or the string 'all' to pin every field
/// (the worst-case policy, matching `intel_pubs` in the pallet benchmarks). The
/// mandatory `mrtd` and `reportdata` are always taken from the quote.
///
/// Handy for tests and for reading the expected values off a known-good quote; in
/// production the expected measurements should be pinned to values you decided on
/// out of band, not copied from the quote being verified.
function tdReportPolicyFromQuote(quote, pin = 'all') {
    const buf = toBuffer(quote, 'quote');
    if (buf.length < QUOTE_HEADER_SIZE + QUOTE_BODY_SIZE) {
        throw new Error(
            `quote: expected at least ${QUOTE_HEADER_SIZE + QUOTE_BODY_SIZE} bytes, ` +
            `got ${buf.length}`,
        );
    }
    const body = buf.subarray(QUOTE_HEADER_SIZE, QUOTE_HEADER_SIZE + QUOTE_BODY_SIZE);

    const read = (field) =>
        Buffer.from(body.subarray(BODY_OFFSETS[field], BODY_OFFSETS[field] + SIZES[field]));

    const pinned = pin === 'all' ? OPTIONAL : pin;
    const unknown = pinned.filter((f) => !OPTIONAL.includes(f));
    if (unknown.length > 0) {
        throw new Error(`cannot pin unknown or mandatory field(s): ${unknown.join(', ')}`);
    }

    const fields = { mrtd: read('mrtd'), reportdata: read('reportdata') };
    for (const field of pinned) {
        fields[field] = read(field);
    }
    return fields;
}

/// Decode a pubs blob back into named fields, with unpinned ones as null. Applies the
/// same validation the on-chain parser does, so it doubles as a check that a blob is
/// canonical.
function parseTdReportPolicy(pubs) {
    const buf = toBytes(pubs, TD_REPORT_POLICY_SIZE, 'pubs');
    if (buf[OFFSETS.version] !== POLICY_VERSION_V1) {
        throw new Error(`unsupported policy version ${buf[OFFSETS.version]}`);
    }
    const bitmap = buf[OFFSETS.bitmap];
    const slice = (field) =>
        Buffer.from(buf.subarray(OFFSETS[field], OFFSETS[field] + SIZES[field]));

    const out = { version: POLICY_VERSION_V1, bitmap };
    for (const field of MANDATORY) {
        out[field] = '0x' + slice(field).toString('hex');
    }
    for (const field of OPTIONAL) {
        const value = slice(field);
        if ((bitmap & BITS[field]) !== 0) {
            out[field] = '0x' + value.toString('hex');
        } else if (value.every((b) => b === 0)) {
            out[field] = null;
        } else {
            throw new Error(`non-canonical policy: ${field} is not pinned but is non-zero`);
        }
    }
    return out;
}

// =============================================================================
// CLI
// =============================================================================

const USAGE = `Build or inspect the pubs blob (canonical TdReportPolicy) for an Intel TDX
submission to the tee verifier.

Usage:
  tee_policy.js from-quote <quote> [--pin <fields>] [--json]
  tee_policy.js build --mrtd <v> --reportdata <v> [--<field> <v> ...] [--json]
  tee_policy.js parse <pubs> [--json]

Commands:
  from-quote  Read the TD report values out of a quote and pin the requested fields.
              For tests and inspection: in production the expected measurements should
              be values decided out of band, not copied from the quote being verified.
  build       Assemble a policy from values given on the command line.
  parse       Decode a pubs blob into named fields, applying the same validation the
              on-chain parser does. Unpinned fields print as null.

Options:
  --pin <fields>  from-quote only: 'all' (default), 'none', or a comma-separated list
                  of optional fields (${OPTIONAL.join(', ')}).
  --json          Print the decoded fields as JSON instead of just the pubs hex.
  -h, --help      Show this help.

Fields for 'build': mrtd (48 bytes) and reportdata (64 bytes) are mandatory; ${OPTIONAL.join(', ')}
are optional (xfam is 8 bytes, the rest 48). Omitted optional fields are left unchecked.

<quote>, <pubs> and every field value may be a hex string (with or without 0x), a path
to a file holding either hex text or raw bytes, or '-' to read stdin.

Examples:
  tee_policy.js from-quote quote.bin > pubs.hex
  tee_policy.js from-quote quote.bin --pin rtmr0,rtmr2 --json
  tee_policy.js build --mrtd 0x91eb... --reportdata 0x9a9d... --xfam 0xe702060000000000
  tee_policy.js parse pubs.hex --json
`;

const HEX_TEXT = /^(0[xX])?[0-9a-fA-F\s]+$/;

/// Resolve a CLI argument to bytes: '-' reads stdin, an existing path reads the file
/// (hex text or raw bytes), anything else is treated as a hex literal.
function readValue(arg, field) {
    const fs = require('fs');
    if (arg === '-') {
        return fromHexOrRaw(fs.readFileSync(0), field);
    }
    if (fs.existsSync(arg) && fs.statSync(arg).isFile()) {
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

function describe(pubs) {
    const parsed = parseTdReportPolicy(pubs);
    return JSON.stringify(
        {
            pubs: '0x' + pubs.toString('hex'),
            version: parsed.version,
            bitmap: '0x' + parsed.bitmap.toString(16).padStart(2, '0'),
            fields: Object.fromEntries(
                [...MANDATORY, ...OPTIONAL].map((f) => [f, parsed[f]]),
            ),
        },
        null,
        2,
    );
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
    let pubs;

    switch (command) {
        case 'from-quote': {
            if (positional.length !== 1) {
                throw new Error("from-quote takes exactly one <quote> argument");
            }
            const spec = flags.pin === undefined ? 'all' : flags.pin;
            let pin;
            if (spec === 'all') {
                pin = 'all';
            } else if (spec === 'none' || spec === '') {
                pin = [];
            } else {
                pin = spec.split(',').map((f) => f.trim()).filter((f) => f !== '');
            }
            pubs = buildTdReportPolicy(
                tdReportPolicyFromQuote(readValue(positional[0], 'quote'), pin),
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
                if (!MANDATORY.includes(name) && !OPTIONAL.includes(name)) {
                    throw new Error(`unknown policy field: --${name}`);
                }
                fields[name] = readValue(value, name);
            }
            pubs = buildTdReportPolicy(fields);
            break;
        }
        case 'parse': {
            if (positional.length !== 1) {
                throw new Error('parse takes exactly one <pubs> argument');
            }
            pubs = toBytes(readValue(positional[0], 'pubs'), TD_REPORT_POLICY_SIZE, 'pubs');
            process.stdout.write(describe(pubs) + '\n');
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

exports.POLICY_VERSION_V1 = POLICY_VERSION_V1;
exports.TD_REPORT_POLICY_SIZE = TD_REPORT_POLICY_SIZE;
exports.POLICY_SIZES = SIZES;
exports.POLICY_OFFSETS = OFFSETS;
exports.POLICY_BITS = BITS;
exports.buildTdReportPolicy = buildTdReportPolicy;
exports.buildTdReportPolicyHex = buildTdReportPolicyHex;
exports.tdReportPolicyFromQuote = tdReportPolicyFromQuote;
exports.parseTdReportPolicy = parseTdReportPolicy;
