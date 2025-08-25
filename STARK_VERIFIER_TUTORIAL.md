# STARK Verifier End-to-End Tutorial

## Overview

This tutorial provides a complete guide for generating STARK proofs using Cairo and submitting them to the zkVerify blockchain using the `stwo` pallet.

## Prerequisites

- Rust and Cargo installed
- Cairo toolchain installed
- Access to a zkVerify node (local or remote)
- Basic knowledge of Substrate/Polkadot

## Step 1: Install Cairo Toolchain

### Install Cairo
```bash
# Install Cairo compiler
curl -L https://github.com/starkware-libs/cairo/releases/latest/download/cairo-x86_64-unknown-linux-gnu.tar.gz | tar -xz
sudo mv cairo/bin/cairo-compile /usr/local/bin/
sudo mv cairo/bin/cairo-run /usr/local/bin/

# Install Python dependencies
pip install cairo-lang
```

### Verify Installation
```bash
cairo-compile --version
cairo-run --version
```

## Step 2: Create a Cairo Program

Create a simple Cairo program that generates a STARK proof:

```cairo
// simple_proof.cairo
%builtins output

from starkware.cairo.common.serialize import serialize_word

func main{output_ptr: felt*}():
    // Simple computation: x^2 + y^2 = z^2
    let x = 3
    let y = 4
    let z = 5
    
    // Verify the computation
    assert x * x + y * y == z * z
    
    // Output the result
    serialize_word(x)
    serialize_word(y)
    serialize_word(z)
    
    return()
end
```

## Step 3: Compile and Run the Cairo Program

```bash
# Compile the program
cairo-compile simple_proof.cairo --output simple_proof_compiled.json

# Run the program to generate proof
cairo-run --program=simple_proof_compiled.json --print_output --print_info
```

## Step 4: Generate STARK Proof

Create a Python script to generate the STARK proof:

```python
# generate_proof.py
import json
from cairo_rs_py import CairoRunner, CompiledProgram

def generate_stark_proof():
    # Load compiled program
    with open('simple_proof_compiled.json', 'r') as f:
        compiled_program = json.load(f)
    
    # Create Cairo runner
    runner = CairoRunner(compiled_program)
    
    # Run the program
    runner.run()
    
    # Generate STARK proof
    proof = runner.generate_proof()
    
    return proof

if __name__ == "__main__":
    proof = generate_stark_proof()
    
    # Save proof to file
    with open('proof.json', 'w') as f:
        json.dump(proof, f, indent=2)
    
    print("STARK proof generated and saved to proof.json")
```

## Step 5: Transform Proof to zkVerify Format

Create a transformation tool to convert Cairo proofs to the format expected by our verifier:

```python
# transform_proof.py
import json
import struct

def transform_cairo_proof_to_zkverify(cairo_proof_path, output_path):
    """
    Transform Cairo proof format to zkVerify format
    """
    with open(cairo_proof_path, 'r') as f:
        cairo_proof = json.load(f)
    
    # Extract components from Cairo proof
    commitments = cairo_proof.get('commitments', [])
    decommitments = cairo_proof.get('decommitments', [])
    fri_proof = cairo_proof.get('fri_proof', {})
    public_inputs = cairo_proof.get('public_inputs', [])
    
    # Transform to zkVerify format
    zkverify_proof = {
        "commitments": [commitment.hex() for commitment in commitments],
        "decommitments": [decommitment.hex() for decommitment in decommitments],
        "fri_proof": {
            "layers": fri_proof.get('layers', [])
        },
        "public_inputs": public_inputs
    }
    
    # Save transformed proof
    with open(output_path, 'w') as f:
        json.dump(zkverify_proof, f, indent=2)
    
    print(f"Proof transformed and saved to {output_path}")

def generate_verification_key():
    """
    Generate verification key for the proof
    """
    vk = {
        "root": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        "params": {
            "alpha": 12345,
            "beta": 67890
        }
    }
    
    with open('verification_key.json', 'w') as f:
        json.dump(vk, f, indent=2)
    
    print("Verification key generated and saved to verification_key.json")

if __name__ == "__main__":
    transform_cairo_proof_to_zkverify('proof.json', 'zkverify_proof.json')
    generate_verification_key()
```

## Step 6: Submit Proof to zkVerify Blockchain

### Option A: Using Polkadot.js Apps

1. **Start your zkVerify node:**
```bash
./target/release/zkv-relay --dev --tmp --rpc-external
```

2. **Open Polkadot.js Apps:**
   - Go to https://polkadot.js.org/apps/
   - Connect to `ws://127.0.0.1:9944`

3. **Navigate to Extrinsics:**
   - Go to **Developer** → **Extrinsics**
   - Select **stwo** pallet
   - Choose **verifyProof** function

4. **Submit the proof:**
   - Load the proof data from `zkverify_proof.json`
   - Load the verification key from `verification_key.json`
   - Submit the transaction

### Option B: Using Substrate API

Create a Python script to submit proofs programmatically:

```python
# submit_proof.py
import json
import asyncio
from substrateinterface import SubstrateInterface, Keypair

async def submit_proof_to_zkverify():
    # Connect to zkVerify node
    substrate = SubstrateInterface(
        url="ws://127.0.0.1:9944",
        ss58_format=251,
        type_registry_preset="substrate-node-template"
    )
    
    # Load proof data
    with open('zkverify_proof.json', 'r') as f:
        proof_data = json.load(f)
    
    with open('verification_key.json', 'r') as f:
        vk_data = json.load(f)
    
    # Encode proof components
    proof_bytes = json.dumps(proof_data).encode()
    vk_bytes = json.dumps(vk_data).encode()
    public_inputs_bytes = json.dumps(proof_data['public_inputs']).encode()
    
    # Create keypair for signing
    keypair = Keypair.create_from_uri("//Alice")
    
    # Submit proof verification
    call = substrate.compose_call(
        call_module='StwoPallet',
        call_function='verify_proof',
        call_params={
            'proof': proof_bytes.hex(),
            'public_inputs': public_inputs_bytes.hex(),
            'vk': vk_bytes.hex()
        }
    )
    
    # Submit extrinsic
    extrinsic = substrate.create_signed_extrinsic(
        call=call,
        keypair=keypair
    )
    
    result = substrate.submit_extrinsic(
        extrinsic=extrinsic,
        wait_for_inclusion=True
    )
    
    print(f"Proof submitted! Block hash: {result['block_hash']}")
    print(f"Transaction hash: {result['extrinsic_hash']}")

if __name__ == "__main__":
    asyncio.run(submit_proof_to_zkverify())
```

## Step 7: Verify the Result

Check the blockchain events to see if your proof was verified:

```python
# check_result.py
import asyncio
from substrateinterface import SubstrateInterface

async def check_verification_result(block_hash):
    substrate = SubstrateInterface(
        url="ws://127.0.0.1:9944",
        ss58_format=251,
        type_registry_preset="substrate-node-template"
    )
    
    # Get block events
    block = substrate.get_block(block_hash=block_hash)
    
    for event in block['events']:
        if event['module_id'] == 'StwoPallet':
            if event['event_id'] == 'ProofVerified':
                is_valid = event['params'][0]['value']
                print(f"Proof verification result: {'VALID' if is_valid else 'INVALID'}")
                return is_valid
    
    print("No verification event found")
    return None

if __name__ == "__main__":
    # Replace with your actual block hash
    block_hash = "0x..."
    asyncio.run(check_verification_result(block_hash))
```

## Step 8: Complete Example Script

Here's a complete script that automates the entire process:

```python
# complete_example.py
import json
import asyncio
import subprocess
from substrateinterface import SubstrateInterface, Keypair

class STARKVerifierTutorial:
    def __init__(self):
        self.substrate = SubstrateInterface(
            url="ws://127.0.0.1:9944",
            ss58_format=251,
            type_registry_preset="substrate-node-template"
        )
        self.keypair = Keypair.create_from_uri("//Alice")
    
    def compile_cairo_program(self, source_file):
        """Compile Cairo program"""
        print("Compiling Cairo program...")
        result = subprocess.run([
            'cairo-compile', source_file, 
            '--output', 'program_compiled.json'
        ], capture_output=True, text=True)
        
        if result.returncode != 0:
            raise Exception(f"Compilation failed: {result.stderr}")
        
        print("✓ Cairo program compiled successfully")
    
    def generate_proof(self):
        """Generate STARK proof"""
        print("Generating STARK proof...")
        # This would integrate with Cairo's proof generation
        # For demo purposes, we create a mock proof
        proof = {
            "commitments": ["0x1234567890abcdef"],
            "decommitments": ["0xfedcba0987654321"],
            "fri_proof": {"layers": [1, 2, 3, 4]},
            "public_inputs": [3, 4, 5]
        }
        
        with open('proof.json', 'w') as f:
            json.dump(proof, f, indent=2)
        
        print("✓ STARK proof generated")
        return proof
    
    def transform_proof(self):
        """Transform proof to zkVerify format"""
        print("Transforming proof to zkVerify format...")
        
        with open('proof.json', 'r') as f:
            proof = json.load(f)
        
        # Transform to our pallet's format
        zkverify_proof = {
            "commitments": proof["commitments"],
            "decommitments": proof["decommitments"],
            "fri_proof": proof["fri_proof"],
            "public_inputs": proof["public_inputs"]
        }
        
        with open('zkverify_proof.json', 'w') as f:
            json.dump(zkverify_proof, f, indent=2)
        
        print("✓ Proof transformed to zkVerify format")
        return zkverify_proof
    
    async def submit_proof(self, proof_data):
        """Submit proof to blockchain"""
        print("Submitting proof to zkVerify blockchain...")
        
        # Encode data
        proof_bytes = json.dumps(proof_data).encode()
        vk_bytes = json.dumps({
            "root": "0x1234567890abcdef",
            "params": {"alpha": 12345, "beta": 67890}
        }).encode()
        inputs_bytes = json.dumps(proof_data['public_inputs']).encode()
        
        # Create call
        call = self.substrate.compose_call(
            call_module='StwoPallet',
            call_function='verify_proof',
            call_params={
                'proof': proof_bytes.hex(),
                'public_inputs': inputs_bytes.hex(),
                'vk': vk_bytes.hex()
            }
        )
        
        # Submit extrinsic
        extrinsic = self.substrate.create_signed_extrinsic(
            call=call,
            keypair=self.keypair
        )
        
        result = self.substrate.submit_extrinsic(
            extrinsic=extrinsic,
            wait_for_inclusion=True
        )
        
        print(f"✓ Proof submitted! Block: {result['block_hash']}")
        return result
    
    async def run_complete_tutorial(self):
        """Run the complete tutorial"""
        print("🚀 Starting STARK Verifier Tutorial\n")
        
        try:
            # Step 1: Compile Cairo program
            self.compile_cairo_program('simple_proof.cairo')
            
            # Step 2: Generate proof
            proof = self.generate_proof()
            
            # Step 3: Transform proof
            zkverify_proof = self.transform_proof()
            
            # Step 4: Submit to blockchain
            result = await self.submit_proof(zkverify_proof)
            
            print("\n🎉 Tutorial completed successfully!")
            print(f"Transaction hash: {result['extrinsic_hash']}")
            
        except Exception as e:
            print(f"\n❌ Tutorial failed: {e}")

if __name__ == "__main__":
    tutorial = STARKVerifierTutorial()
    asyncio.run(tutorial.run_complete_tutorial())
```

## Troubleshooting

### Common Issues

1. **Cairo compilation fails:**
   - Ensure Cairo toolchain is properly installed
   - Check Cairo program syntax

2. **Proof generation fails:**
   - Verify program logic is correct
   - Check that all dependencies are installed

3. **Blockchain submission fails:**
   - Ensure zkVerify node is running
   - Check network connectivity
   - Verify account has sufficient balance

4. **Proof verification fails:**
   - Check proof format matches expected structure
   - Verify public inputs are correct
   - Ensure verification key is valid

### Debugging Tips

- Use `--print_output` flag with `cairo-run` to see program output
- Check blockchain events for detailed error messages
- Use Polkadot.js Apps to inspect transaction details
- Monitor node logs for additional information

## Next Steps

1. **Customize the Cairo program** for your specific use case
2. **Integrate with your application** using the provided Python scripts
3. **Optimize proof generation** for better performance
4. **Add batch verification** for multiple proofs
5. **Implement proof caching** to avoid regeneration

## Support

For issues and questions:
- Check the zkVerify documentation
- Review Cairo documentation at https://www.cairo-lang.org/
- Open an issue in the zkVerify repository

---

**Note:** This tutorial provides a basic example. For production use, ensure proper error handling, security measures, and performance optimizations.
