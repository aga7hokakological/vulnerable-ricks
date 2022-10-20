use anchor_lang::prelude::*;

use crate::error::ErrorCode;

pub const MAX_DELAY_SEC: u32 = 86_400;

#[account]
#[derive(Default)]
pub struct RicksAuction {
    /// The escrow for which we track position
    pub escrow: Pubkey,
}