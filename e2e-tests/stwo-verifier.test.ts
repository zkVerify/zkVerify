import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { KeyringPair } from '@polkadot/keyring/types';
import { expect } from 'chai';

describe('STARK (Cairo) Verifier - stwo Pallet E2E Tests', () => {
  let api: ApiPromise;
  let alice: KeyringPair;
  let bob: KeyringPair;

  before(async () => {
    // Connect to the zkVerify node
    const provider = new WsProvider('ws://127.0.0.1:9944');
    api = await ApiPromise.create({ provider });

    // Setup test accounts
    const keyring = new Keyring({ type: 'sr25519' });
    alice = keyring.addFromUri('//Alice');
    bob = keyring.addFromUri('//Bob');

    // Wait for connection
    await api.isReady;
  });

  after(async () => {
    await api.disconnect();
  });

  describe('Pallet Integration', () => {
    it('should have stwo pallet available', async () => {
      const pallets = await api.query.stwo;
      expect(pallets).to.exist;
    });

    it('should have correct pallet index', async () => {
      // Check that pallet is registered with index 169
      const metadata = await api.rpc.state.getMetadata();
      const pallet = metadata.asLatest.pallets.find(p => p.name.toString() === 'StwoPallet');
      expect(pallet).to.exist;
      expect(pallet?.index.toNumber()).to.equal(169);
    });
  });

  describe('Proof Verification', () => {
    it('should verify valid STARK proof', async () => {
      // Create test proof data
      const proof = {
        commitments: ["0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"],
        decommitments: ["0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321"],
        fri_proof: {
          layers: [1, 2, 3, 4, 5, 6]
        },
        public_inputs: [42, 43, 44]
      };

      const vk = {
        root: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        params: {
          alpha: 12345,
          beta: 67890
        }
      };

      const public_inputs = [42, 43, 44];

      // Encode data
      const proofBytes = api.createType('Bytes', JSON.stringify(proof));
      const vkBytes = api.createType('Bytes', JSON.stringify(vk));
      const inputsBytes = api.createType('Bytes', JSON.stringify(public_inputs));

      // Submit verification
      const tx = api.tx.stwo.verifyProof(proofBytes, inputsBytes, vkBytes);
      
      const result = await tx.signAndSend(alice);
      
      // Wait for inclusion
      await new Promise((resolve) => {
        result.events.forEach(({ event }) => {
          if (event.section === 'stwo' && event.method === 'ProofVerified') {
            const isValid = event.data[0].toJSON();
            expect(isValid).to.be.a('boolean');
            resolve();
          }
        });
      });
    });

    it('should handle invalid proof gracefully', async () => {
      // Create invalid proof data
      const invalidProof = {
        commitments: [],
        decommitments: [],
        fri_proof: {
          layers: []
        },
        public_inputs: []
      };

      const vk = {
        root: "",
        params: {
          alpha: 0,
          beta: 0
        }
      };

      const public_inputs = [42, 43];

      // Encode data
      const proofBytes = api.createType('Bytes', JSON.stringify(invalidProof));
      const vkBytes = api.createType('Bytes', JSON.stringify(vk));
      const inputsBytes = api.createType('Bytes', JSON.stringify(public_inputs));

      // Submit verification
      const tx = api.tx.stwo.verifyProof(proofBytes, inputsBytes, vkBytes);
      
      const result = await tx.signAndSend(bob);
      
      // Should complete without error (but return false)
      await new Promise((resolve) => {
        result.events.forEach(({ event }) => {
          if (event.section === 'stwo' && event.method === 'ProofVerified') {
            const isValid = event.data[0].toJSON();
            expect(isValid).to.be.false;
            resolve();
          }
        });
      });
    });

    it('should handle public input mismatch', async () => {
      // Create proof with specific public inputs
      const proof = {
        commitments: ["0x1234567890abcdef"],
        decommitments: ["0xfedcba0987654321"],
        fri_proof: {
          layers: [1, 2, 3]
        },
        public_inputs: [42, 43] // Proof has [42, 43]
      };

      const vk = {
        root: "0x1234567890abcdef",
        params: {
          alpha: 12345,
          beta: 67890
        }
      };

      const mismatchedInputs = [42, 999]; // But we pass [42, 999]

      // Encode data
      const proofBytes = api.createType('Bytes', JSON.stringify(proof));
      const vkBytes = api.createType('Bytes', JSON.stringify(vk));
      const inputsBytes = api.createType('Bytes', JSON.stringify(mismatchedInputs));

      // Submit verification
      const tx = api.tx.stwo.verifyProof(proofBytes, inputsBytes, vkBytes);
      
      const result = await tx.signAndSend(alice);
      
      // Should fail due to mismatch
      await new Promise((resolve) => {
        result.events.forEach(({ event }) => {
          if (event.section === 'stwo' && event.method === 'ProofVerified') {
            const isValid = event.data[0].toJSON();
            expect(isValid).to.be.false;
            resolve();
          }
        });
      });
    });
  });

  describe('Performance Tests', () => {
    it('should complete verification within time limits', async () => {
      // Create large proof for performance testing
      const largeProof = {
        commitments: Array(10).fill("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"),
        decommitments: Array(10).fill("0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321"),
        fri_proof: {
          layers: Array(20).fill(1)
        },
        public_inputs: Array(100).fill(42)
      };

      const vk = {
        root: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        params: {
          alpha: 12345,
          beta: 67890
        }
      };

      const public_inputs = Array(100).fill(42);

      // Encode data
      const proofBytes = api.createType('Bytes', JSON.stringify(largeProof));
      const vkBytes = api.createType('Bytes', JSON.stringify(vk));
      const inputsBytes = api.createType('Bytes', JSON.stringify(public_inputs));

      // Measure execution time
      const startTime = Date.now();
      
      const tx = api.tx.stwo.verifyProof(proofBytes, inputsBytes, vkBytes);
      const result = await tx.signAndSend(alice);
      
      // Wait for completion
      await new Promise((resolve) => {
        result.events.forEach(({ event }) => {
          if (event.section === 'stwo' && event.method === 'ProofVerified') {
            resolve();
          }
        });
      });

      const endTime = Date.now();
      const executionTime = endTime - startTime;

      // Should complete within 1.5 seconds (1500ms)
      expect(executionTime).to.be.lessThan(1500);
    });

    it('should handle block space limits', async () => {
      // Create proof that approaches block space limits
      const largeCommitment = "0x" + "a".repeat(1000); // 1KB commitment
      const largeProof = {
        commitments: Array(5).fill(largeCommitment),
        decommitments: Array(5).fill(largeCommitment),
        fri_proof: {
          layers: Array(100).fill(1)
        },
        public_inputs: Array(50).fill(42)
      };

      const vk = {
        root: largeCommitment,
        params: {
          alpha: 12345,
          beta: 67890
        }
      };

      const public_inputs = Array(50).fill(42);

      // Encode data
      const proofBytes = api.createType('Bytes', JSON.stringify(largeProof));
      const vkBytes = api.createType('Bytes', JSON.stringify(vk));
      const inputsBytes = api.createType('Bytes', JSON.stringify(public_inputs));

      // Should complete without exceeding block space
      const tx = api.tx.stwo.verifyProof(proofBytes, inputsBytes, vkBytes);
      const result = await tx.signAndSend(bob);
      
      await new Promise((resolve) => {
        result.events.forEach(({ event }) => {
          if (event.section === 'stwo' && event.method === 'ProofVerified') {
            // Should complete successfully
            expect(event.data[0].toJSON()).to.be.a('boolean');
            resolve();
          }
        });
      });
    });
  });

  describe('Event System', () => {
    it('should emit correct events', async () => {
      const proof = {
        commitments: ["0x1234567890abcdef"],
        decommitments: ["0xfedcba0987654321"],
        fri_proof: {
          layers: [1, 2, 3]
        },
        public_inputs: [42, 43]
      };

      const vk = {
        root: "0x1234567890abcdef",
        params: {
          alpha: 12345,
          beta: 67890
        }
      };

      const public_inputs = [42, 43];

      const proofBytes = api.createType('Bytes', JSON.stringify(proof));
      const vkBytes = api.createType('Bytes', JSON.stringify(vk));
      const inputsBytes = api.createType('Bytes', JSON.stringify(public_inputs));

      const tx = api.tx.stwo.verifyProof(proofBytes, inputsBytes, vkBytes);
      const result = await tx.signAndSend(alice);

      let debugParsingEmitted = false;
      let proofVerifiedEmitted = false;

      await new Promise((resolve) => {
        result.events.forEach(({ event }) => {
          if (event.section === 'stwo') {
            if (event.method === 'DebugParsing') {
              debugParsingEmitted = true;
              expect(event.data[0].toJSON()).to.be.a('boolean');
            } else if (event.method === 'ProofVerified') {
              proofVerifiedEmitted = true;
              expect(event.data[0].toJSON()).to.be.a('boolean');
            }
            
            if (debugParsingEmitted && proofVerifiedEmitted) {
              resolve();
            }
          }
        });
      });

      expect(debugParsingEmitted).to.be.true;
      expect(proofVerifiedEmitted).to.be.true;
    });
  });
});
