// Vesting is anchored to the absolute BABE slot number (`SlotNumber` in the runtime),
// which is wall-clock time in `BLOCK_TIME` units, and no longer to the block count.
//
// The two are orders of magnitude apart on any chain (they differ by the genesis slot),
// so a schedule whose `startingBlock` is an absolute slot is a decisive discriminator:
// under the old `System` provider the block number never reaches it and the schedule
// would stay locked forever.

const { init_api, submitExtrinsic, receivedEvents, BlockUntil, BLOCK_TIME } = require('zkv-lib');

const ReturnCode = {
  Ok: 1,
  ErrSlotNotAbsolute: 2,
  ErrSlotNotWallClock: 3,
  ErrVestedTransfer: 4,
  ErrNotFullyLocked: 5,
  ErrVestOther: 6,
  ErrUnlockedMismatch: 7,
};

const TOTAL = 100000000000000000000n;  // 100VFY
const PER_SLOT = 1000000000000000000n; // 1VFY

// How far in the future the schedule starts, and how long we let it vest.
const START_DELAY_SLOTS = 2n;
const VEST_FOR_SLOTS = 3n;
const SLOT_TOLERANCE = 2n;

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

const nowSlot = () => BigInt(Math.floor(Date.now() / BLOCK_TIME));

const clamp = (value, lo, hi) => (value < lo ? lo : value > hi ? hi : value);

/// The amount still locked by the `vesting` lock, as the runtime last wrote it.
async function vestingLock(api, address) {
  const locks = await api.query.balances.locks(address);
  const lock = locks.find((l) => l.id.toHuman().trim() === 'vesting');
  return lock === undefined ? 0n : lock.amount.toBigInt();
}

async function run(nodeName, networkInfo, _args) {
  const api = await init_api(zombie, nodeName, networkInfo);

  const keyring = new zombie.Keyring({ type: 'sr25519' });
  const alice = keyring.addFromUri('//Alice');
  const beneficiary = keyring.addFromUri('//VestingBeneficiary');

  const slot0 = (await api.query.babe.currentSlot()).toBigInt();
  const block0 = BigInt((await api.query.system.number()).toString());
  const wallClock0 = nowSlot();
  console.log(`slot ${slot0}, block ${block0}, wall clock ${wallClock0}`);

  // The slot number is the system date and time
  const skew = slot0 > wallClock0 ? slot0 - wallClock0 : wallClock0 - slot0;
  if (skew > SLOT_TOLERANCE) {
      console.log(`Slot ${slot0} is ${skew} slot(s) away from the system clock ${wallClock0}`);
      return ReturnCode.ErrSlotNotWallClock;
  }

  // A schedule written against a slot in the near future
  const startingSlot = slot0 + START_DELAY_SLOTS;
  const schedule = { locked: TOTAL, perBlock: PER_SLOT, startingBlock: startingSlot };

  let events = await submitExtrinsic(
      api,
      api.tx.vesting.vestedTransfer(beneficiary.address, schedule),
      alice,
      BlockUntil.InBlock,
      (event) => event.section === 'vesting' && event.method === 'VestingCreated',
  );
  if (!receivedEvents(events)) {
      console.log(`Failed to create the vesting schedule`);
      return ReturnCode.ErrVestedTransfer;
  }

  console.log(`=== Created the vested transfer of ${TOTAL} tokens (startingSlot in ${START_DELAY_SLOTS} slots) ===`);

  // It has not started yet: everything is still locked.
  let locked = await vestingLock(api, beneficiary.address);
  if (locked !== TOTAL) {
      console.log(`Locked ${locked} before the schedule starts, expected ${TOTAL}`);
      return ReturnCode.ErrNotFullyLocked;
  }

  let sleep_duration = Number(START_DELAY_SLOTS + VEST_FOR_SLOTS) * BLOCK_TIME;
  console.log(`Sleeping for ${sleep_duration}ms`)
  await sleep(sleep_duration);

  console.log(`Unlocking the first part`)
  events = await submitExtrinsic(
      api,
      api.tx.vesting.vestOther(beneficiary.address),
      alice,
      BlockUntil.InBlock,
      (event) => event.section === 'vesting' && event.method === 'VestingUpdated',
  );
  const vestedAt = nowSlot();
  if (!receivedEvents(events)) {
      console.log(`Failed to vest`);
      return ReturnCode.ErrVestOther;
  }

  const elapsed = vestedAt > startingSlot ? vestedAt - startingSlot : 0n;
  const expected = TOTAL - clamp(elapsed * PER_SLOT, 0n, TOTAL);

  locked = await vestingLock(api, beneficiary.address);
  const drift = locked > expected ? locked - expected : expected - locked;
  console.log(`Locked ${locked} after ${elapsed} slot(s), expected ${expected}`);

  if (drift > SLOT_TOLERANCE * PER_SLOT) {
      console.log(`Locked ${locked} is ${drift} away from the wall-clock expectation ${expected}`);
      return ReturnCode.ErrUnlockedMismatch;
  }

  return ReturnCode.Ok;
}

module.exports = { run }
