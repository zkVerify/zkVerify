const fs = require('fs');
const path = require('path');

function readHex(...parts) {
    return '0x' + fs.readFileSync(path.join(__dirname, '..', '..', ...parts)).toString('hex');
}

const PROOF = readHex('verifiers', 'kimchi', 'src', 'resources', 'generated_4096', 'proof.bin');
const VERIFIER_INDEX = readHex('verifiers', 'kimchi', 'src', 'resources', 'generated_4096', 'verifier_index.bin');

exports.PROOF = PROOF;
exports.PUBS = [];
exports.VK = {
    verifierIndexBytes: VERIFIER_INDEX,
    srsId: 'Vesta16',
};
