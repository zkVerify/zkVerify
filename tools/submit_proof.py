#!/usr/bin/env python3
"""
Proof Submission Tool for zkVerify STARK Verifier

This tool submits STARK proofs to the zkVerify blockchain using the stwo pallet.
"""

import json
import asyncio
import argparse
import sys
from pathlib import Path

try:
    from substrateinterface import SubstrateInterface, Keypair
except ImportError:
    print("❌ Error: substrateinterface not installed. Install with: pip install substrate-interface")
    sys.exit(1)

class ProofSubmitter:
    def __init__(self, node_url="ws://127.0.0.1:9944", ss58_format=251):
        """
        Initialize the proof submitter
        
        Args:
            node_url (str): WebSocket URL of the zkVerify node
            ss58_format (int): SS58 format for the network
        """
        self.node_url = node_url
        self.ss58_format = ss58_format
        self.substrate = None
        self.keypair = None
    
    async def connect(self):
        """Connect to the zkVerify node"""
        try:
            self.substrate = SubstrateInterface(
                url=self.node_url,
                ss58_format=self.ss58_format,
                type_registry_preset="substrate-node-template"
            )
            print(f"✅ Connected to zkVerify node at {self.node_url}")
            return True
        except Exception as e:
            print(f"❌ Failed to connect to node: {e}")
            return False
    
    def load_keypair(self, key_uri="//Alice"):
        """
        Load or create a keypair for signing transactions
        
        Args:
            key_uri (str): Key URI (e.g., "//Alice" for development)
        """
        try:
            self.keypair = Keypair.create_from_uri(key_uri)
            print(f"✅ Loaded keypair: {self.keypair.ss58_address}")
            return True
        except Exception as e:
            print(f"❌ Failed to load keypair: {e}")
            return False
    
    def load_proof_data(self, proof_path, vk_path):
        """
        Load proof and verification key data
        
        Args:
            proof_path (str): Path to proof JSON file
            vk_path (str): Path to verification key JSON file
        """
        try:
            with open(proof_path, 'r') as f:
                proof_data = json.load(f)
            
            with open(vk_path, 'r') as f:
                vk_data = json.load(f)
            
            print(f"✅ Loaded proof from {proof_path}")
            print(f"✅ Loaded verification key from {vk_path}")
            
            return proof_data, vk_data
        except FileNotFoundError as e:
            print(f"❌ File not found: {e}")
            return None, None
        except json.JSONDecodeError as e:
            print(f"❌ Invalid JSON: {e}")
            return None, None
    
    async def submit_proof(self, proof_data, vk_data):
        """
        Submit proof to the blockchain
        
        Args:
            proof_data (dict): Proof data
            vk_data (dict): Verification key data
        """
        if not self.substrate or not self.keypair:
            print("❌ Not connected or keypair not loaded")
            return None
        
        try:
            # Encode proof components
            proof_bytes = json.dumps(proof_data).encode()
            vk_bytes = json.dumps(vk_data).encode()
            public_inputs_bytes = json.dumps(proof_data['public_inputs']).encode()
            
            print("📝 Creating verification call...")
            
            # Submit proof verification
            call = self.substrate.compose_call(
                call_module='SettlementStwoPallet',
                call_function='verify_proof',
                call_params={
                    'proof': proof_bytes.hex(),
                    'public_inputs': public_inputs_bytes.hex(),
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
            
            return result
            
        except Exception as e:
            print(f"❌ Failed to submit proof: {e}")
            return None
    
    async def check_verification_result(self, block_hash):
        """
        Check the verification result from blockchain events
        
        Args:
            block_hash (str): Block hash where the transaction was included
        """
        if not self.substrate:
            print("❌ Not connected to node")
            return None
        
        try:
            # Get block events
            block = self.substrate.get_block(block_hash=block_hash)
            
            print(f"🔍 Checking events in block {block_hash}...")
            
            for event in block['events']:
                if event['module_id'] == 'SettlementStwoPallet':
                    if event['event_id'] == 'ProofVerified':
                        is_valid = event['params'][0]['value']
                        print(f"✅ Proof verification result: {'VALID' if is_valid else 'INVALID'}")
                        return is_valid
                    elif event['event_id'] == 'DebugParsing':
                        parsed = event['params'][0]['value']
                        print(f"📋 Proof parsing: {'SUCCESS' if parsed else 'FAILED'}")
            
            print("⚠️  No verification event found")
            return None
            
        except Exception as e:
            print(f"❌ Failed to check verification result: {e}")
            return None

async def main():
    parser = argparse.ArgumentParser(
        description="Submit STARK proofs to zkVerify blockchain",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s --proof proof.json --vk verification_key.json
  %(prog)s --proof proof.json --vk verification_key.json --check-result
  %(prog)s --node ws://localhost:9944 --proof proof.json --vk verification_key.json
        """
    )
    
    parser.add_argument('--proof', required=True, metavar='PROOF_FILE',
                       help='Path to proof JSON file')
    parser.add_argument('--vk', required=True, metavar='VK_FILE',
                       help='Path to verification key JSON file')
    parser.add_argument('--node', default='ws://127.0.0.1:9944',
                       help='zkVerify node WebSocket URL (default: ws://127.0.0.1:9944)')
    parser.add_argument('--key', default='//Alice',
                       help='Key URI for signing (default: //Alice)')
    parser.add_argument('--check-result', action='store_true',
                       help='Check verification result after submission')
    
    args = parser.parse_args()
    
    # Validate files exist
    if not Path(args.proof).exists():
        print(f"❌ Proof file not found: {args.proof}")
        sys.exit(1)
    
    if not Path(args.vk).exists():
        print(f"❌ Verification key file not found: {args.vk}")
        sys.exit(1)
    
    # Create submitter
    submitter = ProofSubmitter(node_url=args.node)
    
    # Connect to node
    if not await submitter.connect():
        sys.exit(1)
    
    # Load keypair
    if not submitter.load_keypair(args.key):
        sys.exit(1)
    
    # Load proof data
    proof_data, vk_data = submitter.load_proof_data(args.proof, args.vk)
    if not proof_data or not vk_data:
        sys.exit(1)
    
    # Submit proof
    result = await submitter.submit_proof(proof_data, vk_data)
    if not result:
        sys.exit(1)
    
    # Check result if requested
    if args.check_result:
        await submitter.check_verification_result(result['block_hash'])
    
    print("\n🎉 Proof submission completed successfully!")

if __name__ == "__main__":
    asyncio.run(main())
