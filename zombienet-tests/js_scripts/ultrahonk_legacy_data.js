// Legacy ultrahonk data: same raw proof/VK/pubs bytes as V0_84, wrapped in 'Legacy' variant.
// This preserves backward compatibility with pre-versioning statement hashes.
const v084 = require('./ultrahonk_v0_84_data.js');

const ZK_PROOF = {
    'Legacy': {
        'ZK': v084.ZK_PROOF.V0_84.ZK
    }
};

const PLAIN_PROOF = {
    'Legacy': {
        'Plain': v084.PLAIN_PROOF.V0_84.Plain
    }
};

const PUBS = v084.PUBS;

const VKEY = {
    'Legacy': v084.VK.V0_84
};

exports.ZK_PROOF = ZK_PROOF;
exports.PLAIN_PROOF = PLAIN_PROOF;
exports.PUBS = PUBS;
exports.VK = VKEY;
