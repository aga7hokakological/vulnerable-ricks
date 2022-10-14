use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct UserPosition {
    /// The escrow for which we track position
    pub escrow: Pubkey,
    /// The amount of ricks owned by user
    pub ricks_amount: u64,
}

impl UserPosition {
    pub const LEN: usize = 32 + 8;
}