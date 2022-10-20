use anchor_lang::prelude::*;

use crate::error::ErrorCode;

pub const MAX_DELAY_SEC: u32 = 86_400;

#[account]
#[derive(Default)]
pub struct Escrow {
    /// Creator of the Escrow
    pub creator: Pubkey,
    /// Resolver for this Escrow
    pub resolver: Pubkey,
    /// The nft token
    pub nft_token: Pubkey,
    /// Token mint account. The token this market is denominated in
    pub ricks_token_mint: Pubkey,
    /// Ricks token account
    pub ricks_token_account: Pubkey,
    /// Ricks amount of tokens in the escrow
    pub ricks_amount: u64,
    /// Ricks amount of tokens to be issued per day
    pub ricks_per_day: u64,
    /// Escrow start time
    pub start_time: u64,
    /// A flag checking whether the escrow is finalized
    pub finalized: bool,
    /// The bump seed for the ricks token account
    pub ricks_account_bump: u8
}

impl Escrow {
    pub const LEN: usize = 5 * 32 + 3 * 8 + 2 * 1;
}