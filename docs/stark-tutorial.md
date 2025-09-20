# End-to-End STARK Proof Tutorial for zkVerify

This tutorial provides a complete guide for generating, transforming, and submitting STARK proofs to the zkVerify blockchain using the STARK verifier pallet.

## Prerequisites

- Rust toolchain installed
- Node.js and npm installed
- Basic understanding of STARK proofs and Cairo
- Access to a zkVerify node (local or testnet)

## Overview

This tutorial covers:
1. Setting up the development environment
2. Generating a STARK proof using Cairo
3. Transforming the proof for zkVerify
4. Submitting the proof to the blockchain
5. Verifying the proof was accepted

## Step 1: Environment Setup

### Install Dependencies

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js (for JavaScript examples)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install node

# Clone the zkVerify repository
git clone https://github.com/zkVerify/zkVerify.git
cd zkVerify
```

### Build the STARK Verifier

```bash
# Build the STARK verifier pallet
cargo build -p pallet-stwo-verifier

# Build the transformation tool
cd tools/zkv-stark-transform
cargo build --release
```

## Step 2: Generate STARK Proof with Cairo

### Install Cairo

```bash
# Install Cairo compiler
curl -L https://github.com/cairo-lang/cairo/releases/download/v2.4.3/cairo-v2.4.3-x86_64-unknown-linux-gnu.tar.gz | tar -xz
export PATH=$PATH:./cairo-v2.4.3-x86_64-unknown-linux-gnu/bin
```

### Create a Simple Cairo Program

Create `examples/simple_program.cairo`:

```cairo
%builtins output

from starkware.cairo.common.serialize import serialize_word

func main{output_ptr: felt*}():
    let x = 42
    let y = 24
    let result = x + y
    serialize_word(result)
    return ()
end
```

### Compile and Run

```bash
# Compile the Cairo program
cairo-compile examples/simple_program.cairo --output simple_program.json

# Run the program
cairo-run --program=simple_program.json --print_output --print_info
```

### Generate Proof

```bash
# Generate STARK proof
cairo-run \
    --program=simple_program.json \
    --layout=small \
    --print_output \
    --print_info \
    --proof_mode \
    --trace_file=trace.bin \
    --memory_file=memory.bin \
    --program_input=input.json
```

## Step 3: Transform Proof for zkVerify

### Create Input File

Create `input.json`:

```json
{
  "x": 42,
  "y": 24
}
```

### Transform the Proof

```bash
# Use the transformation tool
./target/release/zkv-stark-transform convert-cairo \
    --input cairo_proof.json \
    --output zkv_proof.json \
    --vk verification_key.json
```

### Verify Transformation

```bash
# Validate the transformed proof
./target/release/zkv-stark-transform validate \
    --proof zkv_proof.json \
    --vk verification_key.json
```

## Step 4: Submit Proof to zkVerify Blockchain

### Start Local Node

```bash
# Start a local zkVerify node
cargo run --bin zkv-node -- --dev
```

### Submit Proof via JavaScript

Create `submit_proof.js`:

```javascript
const { ApiPromise, WsProvider } = require('@polkadot/api');
const fs = require('fs');

async function submitProof() {
    // Connect to the local node
    const provider = new WsProvider('ws://127.0.0.1:9944');
    const api = await ApiPromise.create({ provider });

    // Load the proof and VK
    const proof = JSON.parse(fs.readFileSync('zkv_proof.json', 'utf8'));
    const vk = JSON.parse(fs.readFileSync('verification_key.json', 'utf8'));

    // Create the transaction
    const tx = api.tx.settlementStwoPallet.submitProof(
        { Vk: vk },  // Verification key
        proof,       // STARK proof
        { inputs: [66] }  // Public inputs (42 + 24 = 66)
    );

    // Submit the transaction
    const hash = await tx.signAndSend(alice);
    console.log('Transaction submitted:', hash.toHex());

    // Wait for the transaction to be included
    await new Promise(resolve => setTimeout(resolve, 5000));

    // Check if the proof was verified
    const events = await api.query.system.events();
    const proofVerified = events.find(event => 
        event.event.section === 'settlementStwoPallet' &&
        event.event.method === 'ProofVerified'
    );

    if (proofVerified) {
        console.log('✅ Proof verified successfully!');
        console.log('Statement hash:', proofVerified.event.data[0].toString());
    } else {
        console.log('❌ Proof verification failed');
    }

    await api.disconnect();
}

// Run the submission
submitProof().catch(console.error);
```

### Run the Submission

```bash
node submit_proof.js
```

## Step 5: Verify Proof Acceptance

### Check Blockchain State

```bash
# Query the blockchain for proof verification events
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "state_getStorage", "params":["0x..."]}' \
     http://localhost:9933
```

### Monitor Events

```javascript
// Monitor proof verification events
api.query.system.events((events) => {
    events.forEach((record) => {
        const { event } = record;
        if (event.section === 'settlementStwoPallet' && 
            event.method === 'ProofVerified') {
            console.log('Proof verified:', event.data.toString());
        }
    });
});
```

## Step 6: Advanced Usage

### Batch Proof Submission

```javascript
// Submit multiple proofs in a single transaction
const proofs = [
    { vk: vk1, proof: proof1, inputs: inputs1 },
    { vk: vk2, proof: proof2, inputs: inputs2 },
    { vk: vk3, proof: proof3, inputs: inputs3 }
];

const batchTx = api.tx.utility.batchAll(
    proofs.map(p => api.tx.settlementStwoPallet.submitProof(
        { Vk: p.vk }, p.proof, p.inputs
    ))
);

await batchTx.signAndSend(alice);
```

### Register Verification Key

```javascript
// Register VK to save on transaction fees
const registerTx = api.tx.settlementStwoPallet.registerVk(vk);
const hash = await registerTx.signAndSend(alice);

// Use the VK hash for subsequent proofs
const vkHash = await api.query.settlementStwoPallet.verificationKeys(vkHash);
const submitTx = api.tx.settlementStwoPallet.submitProof(
    { Hash: vkHash }, proof, inputs
);
```

## Troubleshooting

### Common Issues

1. **Proof Format Error**: Ensure the proof is properly transformed using the transformation tool
2. **VK Mismatch**: Verify the VK matches the proof
3. **Input Validation**: Check that public inputs are correctly formatted
4. **Node Connection**: Ensure the zkVerify node is running and accessible

### Debug Commands

```bash
# Check proof format
./target/release/zkv-stark-transform validate --proof zkv_proof.json --vk verification_key.json

# Generate test data
./target/release/zkv-stark-transform generate-test-data --output test_data/ --count 5

# Check node logs
tail -f /tmp/zkv-node.log
```

## Performance Considerations

### Proof Size Limits

- Maximum proof size: ~1.48 MB
- Maximum VK size: ~3.2 KB
- Maximum inputs: 64 elements
- Block size limit: 5 MB

### Verification Time

- Typical verification time: ~7.3 ms
- Block execution limit: 1.5 seconds
- Multiple proofs per block: Supported

## Next Steps

1. **Production Deployment**: Deploy to zkVerify testnet/mainnet
2. **Integration**: Integrate with your application
3. **Monitoring**: Set up monitoring for proof verification
4. **Optimization**: Optimize proof generation and submission

## Resources

- [zkVerify Documentation](https://docs.zkverify.org)
- [STARK Verifier Pallet](verifiers/stwo/README.md)
- [Cairo Documentation](https://www.cairo-lang.org/docs/)
- [Substrate Documentation](https://docs.substrate.io/)

## Support

For questions and support:
- GitHub Issues: [zkVerify Issues](https://github.com/zkVerify/zkVerify/issues)
- Discord: [zkVerify Discord](https://discord.gg/zkverify)
- Documentation: [zkVerify Docs](https://docs.zkverify.org)
