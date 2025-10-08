const { hexToU8a } = require('@polkadot/util');
const { Keyring } = require('@polkadot/keyring');


const { init_api, sudoInitClaim, claim, receivedEvents, claimEthereum } = require('zkv-lib');
const ReturnCode = {
  Ok: 1,
  ErrInitClaim: 2,
  ErrClaim: 3,
  ErrClaimedAmount: 4,
  ErrUnsupportedNetwork: 5,
};

const MNEMONIC = 'poverty popular note inform state innocent grant crumble manage tornado primary list';
const CLAIM_AMOUNT = '1000000000000000000'
const INITIAL_BALANCE = '10000000000000000000'

async function run(nodeName, networkInfo, _args) {

  const api = await init_api(zombie, nodeName, networkInfo);
  
  // Signature generated with https://etherscan.io/verifiedSignatures#
  let eth_signature;
  let ss58Prefix;
  const chain = await api.rpc.system.chain();
  if (chain.toString().startsWith("Volta ")) {
      ss58Prefix = 251;
      // Dest address: xpkQVjJtgK2jwbAkNLEPnWxxuxmBkaLtie7oug3WuRBSZXHz8
      eth_signature = '0x3485f6726eb2efee5356f3b5bdd4df6287c70a73e2b3b9b82e54d8df5395421543456d948bff2109db8fc2342bf4cdc424f70c0161091ff83cb68227a32713e81c';
  } else if (chain.toString().startsWith("zkVerify ")) {
      ss58Prefix = 8741;
      // Dest address: ZKZCeKKpTeBwyoDngETfHfdbSFdWSkWRjZqE8N3BTf5ppbBHY
      eth_signature = '0x79bd9a9c63bbbd918b4a125854a449cdde8bb1fb20df372b15fe74319a1b545324fc75981fea5a401fddcfd65bcaad50710c40e191185364ee1baea3dda286321c';
  } else {
      console.log(`Unsupported chain ${chain}, only Volta and zkVerify are supported`);
      return ReturnCode.ErrUnsupportedNetwork;
  }

  // Check token claim pallet account exists
  const palletAddressOption = await api.query.tokenClaim.palletAccountId();
  const palletAddress = palletAddressOption.unwrap();
  const account_balance = (await api.query.system.account(palletAddress))["data"]["free"];
  if (account_balance <= 0) {
    console.log(`Token Claim pallet account doesn't exist`);
    return ReturnCode.ErrInitClaim;
  }

  // Build a keyring and import Alice's credential
  let keyring = new zombie.Keyring({ type: 'sr25519' });
  const alice = keyring.addFromUri('//Alice');

  // Build some different kind of beneficiaries starting from 0 balance
  const beneficiary_sr = keyring.addFromUri('//BeneficiarySR');
  keyring.setSS58Format(ss58Prefix);
  const beneficiary_polkadot_js = keyring.addFromUri(MNEMONIC);

  keyring = new zombie.Keyring({ type: 'ed25519' });
  keyring.setSS58Format(ss58Prefix);
  const beneficiary_ed = keyring.addFromUri('//BeneficiaryED')

  keyring = new zombie.Keyring({ type: 'ecdsa' });
  keyring.setSS58Format(ss58Prefix);
  const beneficiary_ecdsa = keyring.addFromUri('//BeneficiaryEC')

  const beneficiary_eth = '0xCFb405552868d9906DeDCAbe2F387a37E35e9610';

  // Build Beneficiaries Map
  const beneficiaries_map = new Map();
  beneficiaries_map.set({ 'Ethereum': beneficiary_eth }, CLAIM_AMOUNT);
  beneficiaries_map.set({ 'Substrate': beneficiary_sr.address }, CLAIM_AMOUNT);
  beneficiaries_map.set({ 'Substrate': beneficiary_polkadot_js.address }, CLAIM_AMOUNT);
  beneficiaries_map.set({ 'Substrate': beneficiary_ed.address }, CLAIM_AMOUNT);
  beneficiaries_map.set({ 'Substrate': beneficiary_ecdsa.address }, CLAIM_AMOUNT);

  // Begin claim with beneficiary
  const message = "I'm claiming my funds !";
  let events = await sudoInitClaim(alice, beneficiaries_map, INITIAL_BALANCE, message);
  if (!receivedEvents(events)) {
      console.log(`Failed to initialize claim`);
      return ReturnCode.ErrInitClaim;
  }
  console.log("Claim initialized");

  const keys = await api.query.tokenClaim.beneficiaries.keys();
  const beneficiaries = keys.map(({ args: [beneficiaryId] }) => beneficiaryId);
  console.log('all beneficiaries:', beneficiaries.join(', ')); 

  // Claim SR
  let signature = beneficiary_sr.sign(message);
  events = await claim({Sr25519: beneficiary_sr.publicKey}, {Sr25519: signature});
  if (!receivedEvents(events)) {
      console.log(`Failed to claim`);
      return ReturnCode.ErrClaim;
  }

  // Check beneficiary balance
  let balance_beneficiary = (await api.query.system.account(beneficiary_sr.address))["data"]["free"];
  if (balance_beneficiary != CLAIM_AMOUNT) {
      console.log(`Beneficiary SR balance is ${balance_beneficiary}, expected 1000000000000000000`);
      return ReturnCode.ErrClaimedAmount;
  }

  // Claim SR Prefixed (generated via PolkadotJS)
  signature = "0xc89703e8763b08dfedd8e78959d1c1b28138628e58c7e4bc2c0eefd87d796c10e3fdb7bd35ef713d02c49959d3bfb2c559673a70e54156bc71bff5e7541f1e86";;
  events = await claim({Sr25519: beneficiary_polkadot_js.publicKey}, {Sr25519: hexToU8a(signature)});
  if (!receivedEvents(events)) {
      console.log(`Failed to claim`);
      return ReturnCode.ErrClaim;
  }

  // Check beneficiary balance
  balance_beneficiary = (await api.query.system.account(beneficiary_polkadot_js.address))["data"]["free"];
  if (balance_beneficiary != CLAIM_AMOUNT) {
      console.log(`Beneficiary Prefixed SR balance is ${balance_beneficiary}, expected 1000000000000000000`);
      return ReturnCode.ErrClaimedAmount;
  }

  // Claim Ed25519
  signature = beneficiary_ed.sign(message);
  events = await claim({Ed25519: beneficiary_ed.publicKey}, {Ed25519: signature});
  if (!receivedEvents(events)) {
      console.log(`Failed to claim`);
      return ReturnCode.ErrClaim;
  }

  // Check beneficiary balance
  balance_beneficiary = (await api.query.system.account(beneficiary_ed.address))["data"]["free"];
  if (balance_beneficiary != CLAIM_AMOUNT) {
      console.log(`Beneficiary ED balance is ${balance_beneficiary}, expected 1000000000000000000`);
      return ReturnCode.ErrClaimedAmount;
  }

  // Claim ECDSA
  signature = beneficiary_ecdsa.sign(message);
  events = await claim({Ecdsa: beneficiary_ecdsa.publicKey}, {Ecdsa: signature});
  if (!receivedEvents(events)) {
      console.log(`Failed to claim`);
      return ReturnCode.ErrClaim;
  }

  // Check beneficiary balance
  balance_beneficiary = (await api.query.system.account(beneficiary_ecdsa.address))["data"]["free"];
  if (balance_beneficiary != CLAIM_AMOUNT) {
      console.log(`Beneficiary ECDSA balance is ${balance_beneficiary}, expected 1000000000000000000`);
      return ReturnCode.ErrClaimedAmount;
  }

  // Claim Ethereum
  events = await claimEthereum(beneficiary_eth, eth_signature, beneficiary_sr.address);
  if (!receivedEvents(events)) {
      console.log(`Failed to claim`);
      return ReturnCode.ErrClaim;
  }

  // Check beneficiary balance
  balance_beneficiary = (await api.query.system.account(beneficiary_sr.address))["data"]["free"];
  if (balance_beneficiary != '2000000000000000000') {
      console.log(`Beneficiary ETH balance is ${balance_beneficiary}, expected 2000000000000000000`);
      return ReturnCode.ErrClaimedAmount;
  }

  return ReturnCode.Ok;
}

module.exports = { run }
