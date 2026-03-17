use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_2022_extensions::transfer_fee::{
    withdraw_withheld_tokens_from_mint, WithdrawWithheldTokensFromMint,
};
use crate::{errors::TokenFactoryError, state::TokenConfig, constants::TOKEN_CONFIG_SEED};

#[derive(Accounts)]
pub struct HarvestFees<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    /// CHECK: Token-2022 mint account, validated by token_config PDA seeds
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    #[account(
        seeds = [TOKEN_CONFIG_SEED, mint.key().as_ref()],
        bump = token_config.bump,
        has_one = creator @ TokenFactoryError::UnauthorizedCreator,
    )]
    pub token_config: Account<'info, TokenConfig>,

    /// CHECK: Creator's ATA to receive harvested fees, owned by token program
    #[account(mut)]
    pub creator_ata: AccountInfo<'info>,

    pub token_program: Program<'info, Token2022>,
}

pub fn handler(ctx: Context<HarvestFees>) -> Result<()> {
    withdraw_withheld_tokens_from_mint(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            WithdrawWithheldTokensFromMint {
                token_program_id: ctx.accounts.token_program.to_account_info(),
                mint:             ctx.accounts.mint.to_account_info(),
                destination:      ctx.accounts.creator_ata.to_account_info(),
                authority:        ctx.accounts.creator.to_account_info(),
            },
        ),
    )?;

    msg!("Fees harvested to: {}", ctx.accounts.creator_ata.key());
    Ok(())
}
