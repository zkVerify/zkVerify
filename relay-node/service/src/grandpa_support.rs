// Copyright 2024-2026, Horizen Labs, Inc.
// Copyright (C) Parity Technologies (UK) Ltd.

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! zkVerify-specific GRANDPA integration utilities.

use sp_runtime::traits::{Block as BlockT, Header as _, NumberFor};

use crate::HeaderProvider;

/// Returns the block hash of the block at the given `target_number` by walking
/// backwards from the given `current_header`.
pub(super) fn walk_backwards_to_target_block<Block, HP>(
    backend: &HP,
    target_number: NumberFor<Block>,
    current_header: &Block::Header,
) -> Result<(Block::Hash, NumberFor<Block>), sp_blockchain::Error>
where
    Block: BlockT,
    HP: HeaderProvider<Block>,
{
    let mut target_hash = current_header.hash();
    let mut target_header = current_header.clone();

    loop {
        if *target_header.number() < target_number {
            unreachable!(
                "we are traversing backwards from a known block; \
				 blocks are stored contiguously; \
				 qed"
            );
        }

        if *target_header.number() == target_number {
            return Ok((target_hash, target_number));
        }

        target_hash = *target_header.parent_hash();
        target_header = backend
            .header(target_hash)?
            .expect("Header known to exist due to the existence of one of its descendants; qed");
    }
}
