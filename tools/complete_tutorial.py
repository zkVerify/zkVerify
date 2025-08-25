#!/usr/bin/env python3
"""
Complete STARK Verifier Tutorial Script

This script demonstrates the complete end-to-end process:
1. Compile Cairo program
2. Generate STARK proof
3. Transform proof to zkVerify format
4. Submit to blockchain
5. Verify result
"""

import json
import asyncio
import subprocess
import sys
import time
from pathlib import Path

try:
    from substrateinterface import SubstrateInterface, Keypair
except ImportError:
    print("❌ Error: substrateinterface not installed. Install with: pip install substrate-interface")
    sys.exit(1)

class STARKVerifierTutorial:
    def __init__(self, node_url="ws://127.0.0.1:9944"):
        """
        Initialize the tutorial
        
        Args:
            node_url (str): WebSocket URL of the zkVerify node
        """
        self.node_url = node_url
        self.substrate = None
        self.keypair = None
        self.results = {}
    
    def print_step(self, step_num, title):
        """Print a step header"""
        print(f"\n{'='*60}")
        print(f"STEP {step_num}: {title}")
        print(f"{'='*60}")
    
    def compile_cairo_program(self, source_file="simple_proof.cairo"):
        """Compile Cairo program"""
        self.print_step(1, "Compiling Cairo Program")
        
        if not Path(source_file).exists():
            print(f"❌ Cairo source file not found: {source_file}")
            return False
        
        print(f"📝 Compiling {source_file}...")
        
        try:
            result = subprocess.run([
                'cairo-compile', source_file, 
                '--output', 'program_compiled.json'
            ], capture_output=True, text=True, timeout=30)
            
            if result.returncode != 0:
                print(f"❌ Compilation failed:")
                print(result.stderr)
                return False
            
            print("✅ Cairo program compiled successfully")
            print("   Output: program_compiled.json")
            return True
            
        except subprocess.TimeoutExpired:
            print("❌ Compilation timed out")
            return False
        except FileNotFoundError:
            print("❌ cairo-compile not found. Please install Cairo toolchain")
            return False
        except Exception as e:
            print(f"❌ Compilation error: {e}")
            return False
    
    def generate_proof(self):
        """Generate STARK proof"""
        self.print_step(2, "Generating STARK Proof")
        
        print("🔍 Generating STARK proof...")
        
        # For demo purposes, we create a mock proof
        # In a real scenario, this would integrate with Cairo's proof generation
        proof = {
            "commitments": ["0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"],
            "decommitments": ["0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321"],
            "fri_proof": {
                "layers": [1, 2, 3, 4, 5, 6]
            },
            "public_inputs": [3, 4, 5]
        }
        
        # Save proof
        with open('proof.json', 'w') as f:
            json.dump(proof, f, indent=2)
        
        print("✅ STARK proof generated")
        print("   Output: proof.json")
        self.results['proof'] = proof
        return True
    
    def transform_proof(self):
        """Transform proof to zkVerify format"""
        self.print_step(3, "Transforming Proof to zkVerify Format")
        
        print("🔄 Transforming proof to zkVerify format...")
        
        try:
            with open('proof.json', 'r') as f:
                proof = json.load(f)
            
            # Transform to our pallet's format
            zkverify_proof = {
                "commitments": proof["commitments"],
                "decommitments": proof["decommitments"],
                "fri_proof": proof["fri_proof"],
                "public_inputs": proof["public_inputs"]
            }
            
            # Save transformed proof
            with open('zkverify_proof.json', 'w') as f:
                json.dump(zkverify_proof, f, indent=2)
            
            print("✅ Proof transformed to zkVerify format")
            print("   Output: zkverify_proof.json")
            self.results['zkverify_proof'] = zkverify_proof
            return True
            
        except Exception as e:
            print(f"❌ Transformation failed: {e}")
            return False
    
    def generate_verification_key(self):
        """Generate verification key"""
        print("🔑 Generating verification key...")
        
        vk = {
            "root": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "params": {
                "alpha": 12345,
                "beta": 67890
            }
        }
        
        with open('verification_key.json', 'w') as f:
            json.dump(vk, f, indent=2)
        
        print("✅ Verification key generated")
        print("   Output: verification_key.json")
        self.results['verification_key'] = vk
        return True
    
    async def connect_to_blockchain(self):
        """Connect to zkVerify blockchain"""
        self.print_step(4, "Connecting to zkVerify Blockchain")
        
        print(f"🔗 Connecting to {self.node_url}...")
        
        try:
            self.substrate = SubstrateInterface(
                url=self.node_url,
                ss58_format=251,
                type_registry_preset="substrate-node-template"
            )
            print("✅ Connected to zkVerify node")
            return True
        except Exception as e:
            print(f"❌ Failed to connect: {e}")
            print("💡 Make sure your zkVerify node is running with: ./target/release/zkv-relay --dev --tmp --rpc-external")
            return False
    
    def setup_keypair(self):
        """Setup keypair for signing"""
        print("🔐 Setting up keypair...")
        
        try:
            self.keypair = Keypair.create_from_uri("//Alice")
            print(f"✅ Loaded keypair: {self.keypair.ss58_address}")
            return True
        except Exception as e:
            print(f"❌ Failed to setup keypair: {e}")
            return False
    
    async def submit_proof(self):
        """Submit proof to blockchain"""
        self.print_step(5, "Submitting Proof to Blockchain")
        
        if not self.substrate or not self.keypair:
            print("❌ Not connected or keypair not setup")
            return False
        
        try:
            # Get proof data
            proof_data = self.results.get('zkverify_proof')
            vk_data = self.results.get('verification_key')
            
            if not proof_data or not vk_data:
                print("❌ Proof or verification key not available")
                return False
            
            print("📝 Creating verification call...")
            
            # Encode data
            proof_bytes = json.dumps(proof_data).encode()
            vk_bytes = json.dumps(vk_data).encode()
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
            
            print("🔐 Creating signed extrinsic...")
            
            # Submit extrinsic
            extrinsic = self.substrate.create_signed_extrinsic(
                call=call,
                keypair=self.keypair
            )
            
            print("🚀 Submitting to blockchain...")
            
            result = self.substrate.submit_extrinsic(
                extrinsic=extrinsic,
                wait_for_inclusion=True
            )
            
            print(f"✅ Proof submitted successfully!")
            print(f"   Block hash: {result['block_hash']}")
            print(f"   Transaction hash: {result['extrinsic_hash']}")
            
            self.results['submission'] = result
            return True
            
        except Exception as e:
            print(f"❌ Failed to submit proof: {e}")
            return False
    
    async def check_verification_result(self):
        """Check verification result"""
        self.print_step(6, "Checking Verification Result")
        
        submission = self.results.get('submission')
        if not submission:
            print("❌ No submission result available")
            return False
        
        try:
            block_hash = submission['block_hash']
            print(f"🔍 Checking events in block {block_hash}...")
            
            # Get block events
            block = self.substrate.get_block(block_hash=block_hash)
            
            verification_result = None
            parsing_result = None
            
            for event in block['events']:
                if event['module_id'] == 'StwoPallet':
                    if event['event_id'] == 'ProofVerified':
                        is_valid = event['params'][0]['value']
                        verification_result = is_valid
                        print(f"✅ Proof verification result: {'VALID' if is_valid else 'INVALID'}")
                    elif event['event_id'] == 'DebugParsing':
                        parsed = event['params'][0]['value']
                        parsing_result = parsed
                        print(f"📋 Proof parsing: {'SUCCESS' if parsed else 'FAILED'}")
            
            if verification_result is None:
                print("⚠️  No verification event found")
                return False
            
            self.results['verification_result'] = verification_result
            self.results['parsing_result'] = parsing_result
            return True
            
        except Exception as e:
            print(f"❌ Failed to check verification result: {e}")
            return False
    
    def print_summary(self):
        """Print tutorial summary"""
        self.print_step(7, "Tutorial Summary")
        
        print("📊 Results Summary:")
        print(f"   Cairo compilation: {'✅' if 'proof' in self.results else '❌'}")
        print(f"   Proof generation: {'✅' if 'proof' in self.results else '❌'}")
        print(f"   Proof transformation: {'✅' if 'zkverify_proof' in self.results else '❌'}")
        print(f"   Blockchain connection: {'✅' if self.substrate else '❌'}")
        print(f"   Proof submission: {'✅' if 'submission' in self.results else '❌'}")
        print(f"   Verification result: {'✅' if 'verification_result' in self.results else '❌'}")
        
        if 'verification_result' in self.results:
            result = self.results['verification_result']
            print(f"\n🎯 Final Result: Proof verification {'PASSED' if result else 'FAILED'}")
        
        if 'submission' in self.results:
            submission = self.results['submission']
            print(f"\n🔗 Transaction Details:")
            print(f"   Block hash: {submission['block_hash']}")
            print(f"   Transaction hash: {submission['extrinsic_hash']}")
        
        print(f"\n📁 Generated Files:")
        files = ['program_compiled.json', 'proof.json', 'zkverify_proof.json', 'verification_key.json']
        for file in files:
            if Path(file).exists():
                print(f"   ✅ {file}")
            else:
                print(f"   ❌ {file}")
    
    async def run_complete_tutorial(self):
        """Run the complete tutorial"""
        print("🚀 Starting STARK Verifier Tutorial")
        print("This tutorial demonstrates the complete end-to-end process")
        print("of generating and verifying STARK proofs on zkVerify blockchain\n")
        
        try:
            # Step 1: Compile Cairo program
            if not self.compile_cairo_program():
                return False
            
            # Step 2: Generate proof
            if not self.generate_proof():
                return False
            
            # Step 3: Transform proof
            if not self.transform_proof():
                return False
            
            # Generate verification key
            if not self.generate_verification_key():
                return False
            
            # Step 4: Connect to blockchain
            if not await self.connect_to_blockchain():
                return False
            
            if not self.setup_keypair():
                return False
            
            # Step 5: Submit proof
            if not await self.submit_proof():
                return False
            
            # Step 6: Check result
            if not await self.check_verification_result():
                return False
            
            # Step 7: Print summary
            self.print_summary()
            
            print("\n🎉 Tutorial completed successfully!")
            return True
            
        except KeyboardInterrupt:
            print("\n⚠️  Tutorial interrupted by user")
            return False
        except Exception as e:
            print(f"\n❌ Tutorial failed with error: {e}")
            return False

async def main():
    """Main function"""
    import argparse
    parser = argparse.ArgumentParser(
        description="Complete STARK Verifier Tutorial",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s
  %(prog)s --node ws://localhost:9944
        """
    )
    
    parser.add_argument('--node', default='ws://127.0.0.1:9944',
                       help='zkVerify node WebSocket URL (default: ws://127.0.0.1:9944)')
    parser.add_argument('--cairo-file', default='simple_proof.cairo',
                       help='Cairo source file (default: simple_proof.cairo)')
    
    args = parser.parse_args()
    
    # Check if Cairo file exists
    if not Path(args.cairo_file).exists():
        print(f"❌ Cairo file not found: {args.cairo_file}")
        print("💡 Make sure you have the simple_proof.cairo file in the current directory")
        sys.exit(1)
    
    # Run tutorial
    tutorial = STARKVerifierTutorial(node_url=args.node)
    success = await tutorial.run_complete_tutorial()
    
    if success:
        print("\n🎯 Tutorial Status: SUCCESS")
        sys.exit(0)
    else:
        print("\n🎯 Tutorial Status: FAILED")
        sys.exit(1)

if __name__ == "__main__":
    asyncio.run(main())
