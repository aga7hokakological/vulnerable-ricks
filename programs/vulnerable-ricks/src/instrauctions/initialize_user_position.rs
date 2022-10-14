use anchor_lang::prelude::*;
use solana_program::entrypoint::ProgramResult;

use crate::state::{Escrow, UserPosition};

#[derive(Accounts)]
pub struct InitializeUserPosition<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub escrow: Account<'info, Escrow>,
    #[Account(
        init,
        payer = payer,
        seeds = [b"user", user.key().as_ref(), escrow.key().as_ref()],
        bump,
        space = 8 + UserPosition::LEN,
    )]
    pub user_position: Account<'info, UserPosition>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeUserPosition>) -> ProgramResult {
    let user_position: &mut ctx.accounts.user_position;
    user_position.escrow = ctx.accounts.escrow.key();

    Ok(())
}