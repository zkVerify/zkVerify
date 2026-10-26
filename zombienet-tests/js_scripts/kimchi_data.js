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

function readFixture(name) {
    const fixture = ['verifiers', 'kimchi', 'src', 'resources', name];
    return {
        PROOF: readHex(...fixture, 'proof.bin'),
        PUBS: readFields(...fixture, 'pubs.bin'),
        VK: {
            verifierIndexBytes: readHex(...fixture, 'verifier_index.bin'),
            profile: 'Vesta16',
        },
    };
}

const TWO_CHUNK_LOOKUP_RUNTIME_PUBS_64 = readFixture('generated_131072_lookup_runtime_pubs_64');

exports.ONE_CHUNK_SIMPLE_PUBS_0 = readFixture('generated_4096_maxpoly_4096_simple_pubs_0');
exports.TWO_CHUNK_LOOKUP_RUNTIME_PUBS_64 = TWO_CHUNK_LOOKUP_RUNTIME_PUBS_64;
exports.FOUR_CHUNK_ALL_FEATURES_PUBS_0 = readFixture('generated_262144_all_features_pubs_0');
exports.FOUR_CHUNK_ALL_FEATURES_PUBS_1024 = readFixture('generated_262144_all_features_pubs_1024');

exports.PROOF = TWO_CHUNK_LOOKUP_RUNTIME_PUBS_64.PROOF;
exports.PUBS = TWO_CHUNK_LOOKUP_RUNTIME_PUBS_64.PUBS;
exports.VK = TWO_CHUNK_LOOKUP_RUNTIME_PUBS_64.VK;
