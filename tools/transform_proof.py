#!/usr/bin/env python3
"""
Proof Transformation Tool for zkVerify STARK Verifier

This tool transforms Cairo-generated STARK proofs to the format expected
by the zkVerify blockchain's stwo pallet.
"""

import json
import struct
import argparse
import sys
from pathlib import Path

def transform_cairo_proof_to_zkverify(cairo_proof_path, output_path):
    """
    Transform Cairo proof format to zkVerify format
    
    Args:
        cairo_proof_path (str): Path to Cairo proof JSON file
        output_path (str): Path to save transformed proof
    """
    try:
        with open(cairo_proof_path, 'r') as f:
            cairo_proof = json.load(f)
    except FileNotFoundError:
        print(f"❌ Error: Proof file '{cairo_proof_path}' not found")
        return False
    except json.JSONDecodeError:
        print(f"❌ Error: Invalid JSON in '{cairo_proof_path}'")
        return False
    
    # Extract components from Cairo proof
    commitments = cairo_proof.get('commitments', [])
    decommitments = cairo_proof.get('decommitments', [])
    fri_proof = cairo_proof.get('fri_proof', {})
    public_inputs = cairo_proof.get('public_inputs', [])
    
    # Transform to zkVerify format
    zkverify_proof = {
        "commitments": [commitment.hex() if isinstance(commitment, bytes) else str(commitment) 
                       for commitment in commitments],
        "decommitments": [decommitment.hex() if isinstance(decommitment, bytes) else str(decommitment) 
                         for decommitment in decommitments],
        "fri_proof": {
            "layers": fri_proof.get('layers', [])
        },
        "public_inputs": public_inputs
    }
    
    # Save transformed proof
    try:
        with open(output_path, 'w') as f:
            json.dump(zkverify_proof, f, indent=2)
        print(f"✅ Proof transformed and saved to {output_path}")
        return True
    except Exception as e:
        print(f"❌ Error saving transformed proof: {e}")
        return False

def generate_verification_key(output_path="verification_key.json"):
    """
    Generate verification key for the proof
    
    Args:
        output_path (str): Path to save verification key
    """
    vk = {
        "root": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        "params": {
            "alpha": 12345,
            "beta": 67890
        }
    }
    
    try:
        with open(output_path, 'w') as f:
            json.dump(vk, f, indent=2)
        print(f"✅ Verification key generated and saved to {output_path}")
        return True
    except Exception as e:
        print(f"❌ Error generating verification key: {e}")
        return False

def create_mock_proof(output_path="mock_proof.json"):
    """
    Create a mock proof for testing purposes
    
    Args:
        output_path (str): Path to save mock proof
    """
    mock_proof = {
        "commitments": ["0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"],
        "decommitments": ["0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321"],
        "fri_proof": {
            "layers": [1, 2, 3, 4, 5, 6]
        },
        "public_inputs": [3, 4, 5]
    }
    
    try:
        with open(output_path, 'w') as f:
            json.dump(mock_proof, f, indent=2)
        print(f"✅ Mock proof created and saved to {output_path}")
        return True
    except Exception as e:
        print(f"❌ Error creating mock proof: {e}")
        return False

def validate_proof_format(proof_path):
    """
    Validate that a proof has the correct format for zkVerify
    
    Args:
        proof_path (str): Path to proof file to validate
    """
    try:
        with open(proof_path, 'r') as f:
            proof = json.load(f)
    except Exception as e:
        print(f"❌ Error reading proof file: {e}")
        return False
    
    required_fields = ['commitments', 'decommitments', 'fri_proof', 'public_inputs']
    missing_fields = [field for field in required_fields if field not in proof]
    
    if missing_fields:
        print(f"❌ Missing required fields: {missing_fields}")
        return False
    
    if not isinstance(proof['fri_proof'], dict) or 'layers' not in proof['fri_proof']:
        print("❌ Invalid FRI proof format")
        return False
    
    print("✅ Proof format is valid")
    return True

def main():
    parser = argparse.ArgumentParser(
        description="Transform Cairo proofs to zkVerify format",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s --transform proof.json zkverify_proof.json
  %(prog)s --generate-vk
  %(prog)s --create-mock
  %(prog)s --validate proof.json
        """
    )
    
    parser.add_argument('--transform', nargs=2, metavar=('INPUT', 'OUTPUT'),
                       help='Transform Cairo proof to zkVerify format')
    parser.add_argument('--generate-vk', action='store_true',
                       help='Generate verification key')
    parser.add_argument('--create-mock', action='store_true',
                       help='Create mock proof for testing')
    parser.add_argument('--validate', metavar='PROOF_FILE',
                       help='Validate proof format')
    
    args = parser.parse_args()
    
    if not any([args.transform, args.generate_vk, args.create_mock, args.validate]):
        parser.print_help()
        return
    
    success = True
    
    if args.transform:
        success &= transform_cairo_proof_to_zkverify(args.transform[0], args.transform[1])
    
    if args.generate_vk:
        success &= generate_verification_key()
    
    if args.create_mock:
        success &= create_mock_proof()
    
    if args.validate:
        success &= validate_proof_format(args.validate)
    
    if success:
        print("\n🎉 All operations completed successfully!")
        sys.exit(0)
    else:
        print("\n❌ Some operations failed!")
        sys.exit(1)

if __name__ == "__main__":
    main()
