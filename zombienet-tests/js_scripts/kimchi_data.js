const fs = require('fs');
const path = require('path');

function readHex(...parts) {
    return '0x' + fs.readFileSync(path.join(__dirname, '..', '..', ...parts)).toString('hex');
}

function readFields(...parts) {
    const bytes = fs.readFileSync(path.join(__dirname, '..', '..', ...parts));
    if (bytes.length % 32 !== 0) {
        throw new Error('Kimchi public input fixture must contain 32-byte fields');
    }
    return Array.from({ length: bytes.length / 32 }, (_, i) =>
        '0x' + bytes.subarray(i * 32, (i + 1) * 32).toString('hex')
    );
}

const FIXTURE = ['verifiers', 'kimchi', 'src', 'resources', 'generated_131072_lookup_runtime_pubs_64'];
const PROOF = readHex(...FIXTURE, 'proof.bin');
const PUBS = readFields(...FIXTURE, 'pubs.bin');
const VERIFIER_INDEX = readHex(...FIXTURE, 'verifier_index.bin');

exports.PROOF = PROOF;
exports.PUBS = PUBS;
exports.VK = {
    verifierIndexBytes: VERIFIER_INDEX,
    profile: 'Vesta16',
};
