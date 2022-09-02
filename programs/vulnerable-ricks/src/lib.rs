use anchor_lang::{
    prelude::*, prelude::*, solana_program::program::invoke, solana_program::system_instruction,
};
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::{self, SetAuthority, Token, TokenAccount, Transfer};
use spl_token::instruction::AuthorityType;

pub const DAY: u64 = 24 * 60 * 60;
const LAMPORTS_BUFFER: u64 = 20_000_000;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod ricks {
    use super::*;

    // const NFT_ESCROW_PDA_SEED: &[u8] = b"NFTEscrow";
    pub fn initialize_nft_escrow(ctx: Context<InitializeNFTEscrow>, ricks_token_amount: u64) -> Result<()> {
        let (nft_escrow, bump_seed) = Pubkey::find_program_address(
            &[&ctx.accounts.nft_escrow_account.to_account_info().key.to_bytes()],
            &ctx.program_id,
        );

        let _ = &ctx.accounts.validate_function_accounts(nft_escrow)?;

        let seeds = &[
            &ctx.accounts.nft_escrow_account.to_account_info().key.to_bytes(),
            &[bump_seed][..],
        ];

        let nft_escrow_account = &mut ctx.accounts.nft_escrow_account;

        if ricks_token_amount <= 0 {
            return Err(NFTEscrowError::RicksCountGreaterThanZero.into());
        }

        let nft_owner = &mut ctx.accounts.nft_owner.key();

        nft_escrow_account.is_initialized = true;
        nft_escrow_account.nft_owner = *nft_owner;
        nft_escrow_account.nft_deposit = ctx.accounts.nft_deposit.key();
        nft_escrow_account.nft_fraction_token_amount = ricks_token_amount;
        nft_escrow_account.nft_deposit_time = Clock::get().unwrap().unix_timestamp;

        let ricks_token_cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.clone(),
            token::MintTo {
                mint: ctx.accounts.nft_ricks_fraction_tokens.to_account_info(),
                to: ctx.accounts.escrow_ricks_token_account.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        );
        token::mint_to(
            ricks_token_cpi_ctx, 
            ricks_token_amount
        )?;

        Ok(())
    }

    pub fn withdraw_nft(ctx: Context<WithdrawNFT>) -> Result<()> {
        let (nft_escrow, bump_seed) = Pubkey::find_program_address(
            &[&ctx.accounts.nft_escrow_account.to_account_info().key.to_bytes()],
            &ctx.program_id,
        );

        let seeds = &[
            &ctx.accounts.nft_escrow_account.to_account_info().key.to_bytes(),
            &[bump_seed][..],
        ];

        let nft_owner = &mut ctx.accounts.nft_owner.key();
        let nft_escrow_account = &mut ctx.accounts.nft_escrow_account;

        if nft_escrow_account.is_initialized {
            return Err(WithdrawalError::CannotWithdraw.into())
        }

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.nft_escrow_account.to_account_info(),
                to: ctx.accounts.nft_owner.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, 1)?;

        Ok(())
    }

    /// Creates and initialize a new auction
    pub fn initialize_auction(
        ctx: Context<InitializeAuction>,
        ricks_per_day: u64,
        // bidding_start_time: i64,
        // bidding_end_time: i64,
    ) -> Result<()> {
        let nft_escrow_account = &mut ctx.accounts.nft_escrow_account;
        let initialized = nft_escrow_account.is_initialized;

        let bidding_start_time = nft_escrow_account.nft_deposit_time + DAY as i64;

        let auction_state = &mut ctx.accounts.auction_state;
        let clock = &ctx.accounts.clock;

        auction_state.nft_auction_time_update = nft_escrow_account.nft_deposit_time;

        if !initialized {
            msg!("Cannot initialize auction");
            return Err(AuctionError::CannotInitializeAuction.into());
        };

        if nft_escrow_account.nft_fraction_token_amount == 0 {
            return Err(AuctionError::AllRicksAreAuctioned.into());
        }

        auction_state.initializer = *ctx.accounts.initializer.key;

        //Check that start time is >= current time and that end time is after the start time
        if auction_state.nft_auction_time_update != nft_escrow_account.nft_deposit_time {
            auction_state.bidding_start_time = auction_state.nft_auction_time_update + DAY as i64; // bidding start time;
            auction_state.bidding_end_time = bidding_start_time + DAY as i64;
            auction_state.nft_auction_time_update = auction_state.bidding_end_time;
        } else {
            auction_state.bidding_start_time = auction_state.nft_auction_time_update + DAY as i64;
            auction_state.bidding_end_time = bidding_start_time + DAY as i64;
            auction_state.nft_auction_time_update = auction_state.bidding_end_time;
        }

        if auction_state.bidding_end_time <= auction_state.bidding_start_time {
            msg!("End time invalid");
            return Err(AuctionError::EndingTimeTooEarly.into());
        };

        auction_state.beneficiary = *ctx.accounts.beneficiary.key;
        auction_state.highest_bid_address = None;
        auction_state.highest_bid_amount = None;
        auction_state.ricks_per_day = ricks_per_day;
        auction_state.ended_funds_transferred = false;
        auction_state.bump = *ctx.bumps.get("auction_state").unwrap();

        Ok(())
    }

    /// Bid
    #[access_control(valid_bid_time(&ctx))]
    pub fn bid(ctx: Context<MakeBid>, amount: u64) -> Result<()> {
        let auction_state = &mut ctx.accounts.auction_state;
        let highest_bid = auction_state.highest_bid_amount;

        // Check if bid is higher than the highest bid
        if highest_bid.is_some() && amount <= highest_bid.unwrap() {
            return Err(AuctionError::BidTooLow.into());
        };

        // Transfer lamports from signer/ bidder to bid account
        let bidder = &mut ctx.accounts.bidder;
        let bid_account = &mut ctx.accounts.bid_account;
        let lamports_in_bid_account = bid_account.to_account_info().lamports();
        let total_lamports_needed = amount.checked_add(LAMPORTS_BUFFER).unwrap(); // TODO: Better error handling here

        // Check if amount in bid account is enough, if not, then transfer additional lamports from the bidder address to the bid account
        if lamports_in_bid_account < total_lamports_needed {
            let transfer_amount = total_lamports_needed
                .checked_sub(lamports_in_bid_account)
                .unwrap();

            let transfer_from_bidder_instruction =
                system_instruction::transfer(bidder.key, &bid_account.key(), transfer_amount);
            let account_infos = [bidder.to_account_info(), bid_account.to_account_info()];

            invoke(&transfer_from_bidder_instruction, &account_infos)?;
        }

        // Assert bid_account has enough lamports to fulfill the bid
        let lamports_in_bid_account = bid_account.to_account_info().lamports();
        assert!(lamports_in_bid_account >= amount.checked_add(LAMPORTS_BUFFER).unwrap());

        // Write the highest bid address and amount to auction state account
        auction_state.highest_bid_address = Some(*bidder.as_ref().key);
        auction_state.highest_bid_amount = Some(amount);

        // Write data to bid PDA
        bid_account.bidder = *bidder.key;
        bid_account.amount = amount;
        bid_account.auction = auction_state.key();
        bid_account.bump = *ctx.bumps.get("bid_account").unwrap();

        Ok(())
    }

    /// After an auction ends (determined by `bidding_end_time`), anyone can initiate end_auction
    /// which will transfer the highest bid from the bid account to the beneficiary listed in the auction state account
    #[access_control(end_auction_time_valid(&ctx.accounts.auction_state, &ctx.accounts.clock))]
    pub fn end_auction(ctx: Context<EndAuction>) -> Result<()> {
        let nft_escrow_account = &mut ctx.accounts.nft_escrow_account;

        let auction_state = &mut ctx.accounts.auction_state;
        let ricks = auction_state.ricks_per_day;
        let bid_account = &mut ctx.accounts.bid_account;
        let beneficiary = &mut ctx.accounts.beneficiary;

        if auction_state.ended_funds_transferred {
            return Err(AuctionError::AuctionAlreadyEnded.into());
        };

        if auction_state.key() != bid_account.auction {
            return Err(AuctionError::AccountMismatch.into());
        }

        if *beneficiary.key != auction_state.beneficiary {
            return Err(AuctionError::InvalidBeneficiary.into());
        };

        let highest_bid_address = auction_state
            .highest_bid_address
            .ok_or(AuctionError::NoBids)?;
        if bid_account.bidder != highest_bid_address {
            return Err(AuctionError::AccountMismatch.into());
        };

        let transfer_amount = auction_state
            .highest_bid_amount
            .ok_or(AuctionError::NoBids)?;
        msg!("Begin first transfer");

        let bid_account_info = bid_account.to_account_info();
        **bid_account_info.try_borrow_mut_lamports()? = bid_account_info
            .lamports()
            .checked_sub(transfer_amount)
            .unwrap();
        **beneficiary.try_borrow_mut_lamports()? =
            beneficiary.lamports().checked_add(transfer_amount).unwrap();

        let index = nft_escrow_account.nft_fraction_owners.iter().position(|x| x.owner == beneficiary.key()).unwrap();

        let owner = Owner { 
            owner: beneficiary.key(),
            nft_of_fraction: nft_escrow_account.nft_deposit.key(),
            fractions_own: nft_escrow_account.nft_fraction_owners[index].fractions_own + ricks,
        };

        nft_escrow_account.nft_fraction_owners.push(owner);
        let remaining_ricks_amt = nft_escrow_account.nft_fraction_token_amount.checked_sub(ricks);
        nft_escrow_account.nft_fraction_token_amount = remaining_ricks_amt.unwrap();

        msg!("First transfer complete");

        auction_state.ended_funds_transferred = true;

        //Close bid_account PDA
        let bid_account_lamports = bid_account_info.lamports();
        **bid_account_info.try_borrow_mut_lamports()? = bid_account_lamports
            .checked_sub(bid_account_lamports)
            .unwrap();
        let bidder_account_info = &mut ctx.accounts.bidder.to_account_info();
        **bidder_account_info.try_borrow_mut_lamports()? = bidder_account_info
            .lamports()
            .checked_add(bid_account_lamports)
            .unwrap();

        Ok(())
    }

    pub fn cancel_nft_escrow(ctx: Context<CancelNFTEscrow>) -> Result<()> {
        let nft_escrow_account = &mut ctx.accounts.nft_escrow_account;

        let nft_owner = &mut ctx.accounts.nft_owner.key();

        if nft_escrow_account.nft_owner != *nft_owner {
            return Err(CancelNFTEscrowError::WrongNFTOwner.into());
        }

        if nft_escrow_account.is_initialized {
            nft_escrow_account.is_initialized = false;
        }

        Ok(())
    }
}


#[derive(Accounts)]
pub struct InitializeNFTEscrow<'info> {
    // A PDA (seed: escrow account's pubkey)
    pub authority: AccountInfo<'info>,
    #[account(signer, zero)]
    pub nft_escrow_account: Account<'info, NFTEscrow>,
    #[account(mut)]
    pub nft_owner: Signer<'info>,
    #[account(mut)]
    pub nft_deposit: Account<'info, TokenAccount>,
    #[account(mut)]
    pub escrow_ricks_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub nft_ricks_fraction_tokens: Account<'info, TokenAccount>,
    pub token_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeNFTEscrow<'info> {
    fn validate_function_accounts(&self, authority: Pubkey) -> Result<()> {
        if *self.authority.key != authority {
            return Err(NFTEscrowError::InvalidProgramAddress.into());
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CancelNFTEscrow<'info> {
    #[account(mut)]
    pub nft_owner: Signer<'info>,
    pub nft_escrow_account: Account<'info, NFTEscrow>,
}

#[derive(Accounts)]
pub struct WithdrawNFT<'info> {
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub nft_owner: Signer<'info>,
    #[account(mut)]
    pub nft_escrow_account: Account<'info, NFTEscrow>,
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct InitializeAuction<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        init,
        payer = initializer,
        space = 8 + 32 + 8 + 8 + 32 + 33 + 9 + 1 + 1, //Note: Leading 8 is for the discriminant
        seeds = [
            b"auction-state",
            initializer.key().as_ref()
        ],
        bump
    )]
    pub auction_state: Account<'info, AuctionState>,
    pub nft_escrow_account: Account<'info, NFTEscrow>,
    /// CHECK: TBD
    pub beneficiary: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Bid {
    bidder: Pubkey,
    amount: u64,
    auction: Pubkey,
    bump: u8,
}

#[derive(Accounts)]
pub struct MakeBid<'info> {
    #[account(mut)]
    pub bidder: Signer<'info>,
    #[account(
        init,
        payer = bidder,
        space = 8 + 32 + 8 + 32 + 1, //Note: Leading 8 is for the discriminant
        seeds = [
            b"bid",
            bidder.key().as_ref(),
            auction_state.key().as_ref()
        ],
        bump
    )]
    pub bid_account: Account<'info, Bid>,
    #[account(
        mut,
        seeds = [
            b"auction-state",
            auction_state.initializer.as_ref()
        ],
        bump = auction_state.bump
    )]
    pub auction_state: Account<'info, AuctionState>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EndAuction<'info> {
    #[account(
        mut,
        seeds = [
            b"auction-state",
            auction_state.initializer.as_ref()
        ],
        bump = auction_state.bump
    )]
    pub auction_state: Account<'info, AuctionState>,
    pub nft_escrow_account: Account<'info, NFTEscrow>,
    #[account(
        mut,
        seeds = [
            b"bid",
            bid_account.bidder.as_ref(),
            auction_state.key().as_ref(), //Note: This should ensure that the bid_account PDA corresponds to the auction_state PDA above, right?
        ],
        bump = bid_account.bump
    )]
    pub bid_account: Account<'info, Bid>,
    #[account(mut, constraint = *bidder.key == bid_account.bidder)]
    /// CHECK: performed above but getting error during anchor build
    pub bidder: AccountInfo<'info>,
    #[account(mut, constraint = *beneficiary.key == auction_state.beneficiary)]
    /// CHECK: performed above but getting error during anchor build
    pub beneficiary: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct AuctionState {
    initializer: Pubkey,
    bidding_start_time: i64,
    bidding_end_time: i64,
    nft_auction_time_update: i64,
    ricks_per_day: u64,
    beneficiary: Pubkey,
    highest_bid_address: Option<Pubkey>,
    highest_bid_amount: Option<u64>,
    ended_funds_transferred: bool,
    bump: u8,
}

#[account]
pub struct NFTEscrow {
    pub escrow_id: u8,
    pub is_initialized: bool,
    pub nft_escrow: Pubkey,
    pub bump_seed: u8,
    pub nft_owner: Pubkey,
    pub nft_deposit: Pubkey,
    pub nft_deposit_time: i64,
    pub nft_fraction_token: Pubkey,
    pub nft_fraction_token_amount: u64,
    pub nft_fraction_owners: Vec<Owner>,
}

#[account]
pub struct Owner {
    pub owner: Pubkey,
    pub nft_of_fraction: Pubkey,
    pub fractions_own: u64,
}

impl<'info> From<&mut InitializeNFTEscrow<'info>> 
    for CpiContext<'_, '_, '_, 'info, SetAuthority<'info>>
{
    fn from(accounts: &mut InitializeNFTEscrow<'info>) -> Self {
        let cpi_accounts = SetAuthority {
            account_or_mint: accounts
                .nft_escrow_account
                .to_account_info()
                .clone(),
            current_authority: accounts.nft_owner.to_account_info().clone(), 
        };
        let cpi_program = accounts.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

fn valid_bid_time(ctx: &Context<MakeBid>) -> Result<()> {
    let auction_state = &ctx.accounts.auction_state;
    let clock = &ctx.accounts.clock;

    if auction_state.bidding_start_time > clock.unix_timestamp {
        return Err(AuctionError::BidTooEarly.into());
    }

    if clock.unix_timestamp > auction_state.bidding_end_time {
        return Err(AuctionError::BidTooLate.into());
    }

    Ok(())
}

fn end_auction_time_valid(
    auction_state: &Account<AuctionState>,
    clock: &Sysvar<Clock>,
) -> Result<()> {
    if auction_state.bidding_end_time > clock.unix_timestamp {
        return Err(AuctionError::AuctionNotOver.into());
    }

    Ok(())
}


#[error_code]
pub enum AuctionError {
    #[msg("Start Time must be greater than or equal to the current time")]
    StartTimeTooEarly,
    #[msg("End Time must be greater than the Start Time")]
    EndingTimeTooEarly,
    #[msg("Bid must be greater than the current highest bid")]
    BidTooLow,
    #[msg("Auction has already ended and funds have been transferred")]
    AuctionAlreadyEnded,
    #[msg("Bid account does not correspond to the correct auction account")]
    AccountMismatch,
    #[msg("Auction had no bids")]
    NoBids,
    #[msg("Beneficiary account provided does not match the auction state account's beneficiary")]
    InvalidBeneficiary,
    #[msg("Bidder on Bid Account does not match highest bidder on auction account")]
    IncorrectBidAccount,
    #[msg("Cannot refund bid prior to auction end and settling of winning bid")]
    InvalidRefund,
    #[msg("The highest bid in the auction cannot be refunded")]
    HighestBidderCannotRefund,
    #[msg("Bids can only be submitted after the auction has begun")]
    BidTooEarly,
    #[msg("Bids can only be submitted before the auction ends")]
    BidTooLate,
    #[msg("Cannot end auction before auction end time elapses")]
    AuctionNotOver,
    #[msg("Cannot initialize auction")]
    CannotInitializeAuction,
    #[msg("All Ricks are auctioned")]
    AllRicksAreAuctioned,
}

#[error_code]
pub enum NFTEscrowError {
    #[msg("Rick Count Should be greater than zero")]
    RicksCountGreaterThanZero,
    #[msg("Invalid Owner")]
    InvalidOwner,
    #[msg("Invalid Program Address")]
    InvalidProgramAddress,
}

#[error_code]
pub enum WithdrawalError {
    #[msg("Cannot withdraw because of initialization")]
    CannotWithdraw,
}

#[error_code]
pub enum CancelNFTEscrowError {
    #[msg("Wrong NFT Owner")]
    WrongNFTOwner,
}