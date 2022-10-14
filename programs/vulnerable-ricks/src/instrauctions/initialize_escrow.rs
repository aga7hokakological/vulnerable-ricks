use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use solana_program::entrypoint::ProgramResult;

use crate::state::{Escrow};

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct InitializeEscrow {
    /// The nft token which is to be staked
    pub nft_token: Pubkey,
    /// The number of ricks
    pub ricks_amount: u64,
    /// resolver of this escrow
    pub resolver: Pubkey,
}

#[derive(Accounts)]
#[instruction(params: InitializeEscrow)]
pub struct InitializeEscrow<'info> {
    /// The market account to initialize
    #[account(init, payer = creator)]
    pub escrow: Account<'info, Escrow>,
    /// The authority for the token account
    #[account(seeds = [b"authority", escrow.key_ref().as_ref()], bump)]
    pub authority: AccountInfo<'info>,
    /// Creator of the escrow
    pub creator: Signer<'info>,
    /// The token that this escrow is denominated in
    pub token_mint: Box<Account<'info, Mint>>,
    /// Escrow for the ricks token account
    #[account(
        init,
        payer = creator,
        token::mint = token_mint,
        token::authority = authority,
        seeds = [b"ricks", escrow.key_ref().as_ref()], 
        bump,
    )]
    pub ricks_token_account: Box<Account<'info, TokenAccount>>,
    /// The solana system program
    pub system_program: Program<'info, System>,
    /// The SPL token program
    pub token_program: Program<'info, Token>,
    /// The Sysvar rent
    pub rent: Sysvar<'info, Rent>
}

impl InitializeEscrow<'_> {
    pub fn validate_params(&self, ricks_amount: u64) -> Result<()> {
        if ricks_amount == 0 {
            return Err(error!(ErrorCode::RicksAmountCannotBeZero));
        }

        Ok(())
    }
} 