// Understanding RICKS: https://www.paradigm.xyz/2021/10/ricks


use anchor_lang::{
    prelude::*, prelude::*, solana_program::program::invoke, solana_program::system_instruction,
};
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::{self, SetAuthority, Token, TokenAccount, Transfer, Mint};
use spl_token::instruction::AuthorityType;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod ricks {
    use super::*;

}

