use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use solana_program::entrypoint::ProgramResult;

use common::traits::KeyRef;

use crate::error::ErrorCode;
use crate::state::{Escrow, Outcome, UserPosition};
use crate::utils::signer_transfer;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    /// The user withdrawing funds from escrow account
    pub user: Signer<'info>,
    /// The yes token account for the market.
    /// 
    /// CHECK: We do not read any data from this account. The correctness of the
    /// account is checked by the constraint on the market account. Writes
    /// only occur via the token program, which performs necessary checks on
    /// sufficient balance and matching token mints.
    #[account(mut)]
    pub ricks_token_account: UncheckedAccount<'info>,
    /// The user's token account. We explicitly check the owner for this account
    #[account(
        mut, 
        constraint = user_token_account.key_ref() != ricks_token_account.key_ref() @ ErrorCode::UserAccountCannotBeEscrowAccount,
        constraint = user_token_account.owner == *user.key_ref() @ ErrorCode::UserAccountIncorrectOwner,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    /// The authority for the market token accounts.
    /// 
    /// CHECK: We do not read/write any data from this account.
    #[account(seeds = [b"authority", market.key_ref().as_ref()], bump)]
    pub authority: AccountInfo<'info>,
    /// The Escrow account
    #[account(
        mut,
        // constraint = escrow.outcome == O
        has_one = ricks_token_account @ ErrorCode::IncorrectRicksEscrow,
    )]
    pub escrow: Account<'info, Escrow>,
    /// The user's position account [UserPosition] for this market
    #[account(mut, seeds = [b"user", user.key_ref().as_ref(), escrow.key_ref().as_ref()], bump)]
    pub user_position: Account<'info, UserPosition>,
    /// The SPL Token Program
    pub token_program: Program<'info, Token>,
}

impl Withdraw<'_> {
    fn 
}