use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use crate::{errors::TokenFactoryError, state::TokenConfig, constants::TOKEN_CONFIG_SEED};

#[derive(Accounts)]
pub struct FreezeTokenAccount<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    /// CHECK: This is the Token-2022 mint account, validated by token_config seeds
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    /// CHECK: This is the target token account to freeze, owned by token program
    #[account(mut)]
    pub target_account: AccountInfo<'info>,

    #[account(
        seeds = [TOKEN_CONFIG_SEED, mint.key().as_ref()],
        bump = token_config.bump,
        has_one = creator @ TokenFactoryError::UnauthorizedCreator,
    )]
    pub token_config: Account<'info, TokenConfig>,

    pub token_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct ThawTokenAccount<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    /// CHECK: This is the Token-2022 mint account, validated by token_config seeds
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    /// CHECK: This is the target token account to thaw, owned by token program
    #[account(mut)]
    pub target_account: AccountInfo<'info>,

    #[account(
        seeds = [TOKEN_CONFIG_SEED, mint.key().as_ref()],
        bump = token_config.bump,
        has_one = creator @ TokenFactoryError::UnauthorizedCreator,
    )]
    pub token_config: Account<'info, TokenConfig>,

    pub token_program: Program<'info, Token2022>,
}

pub fn handler(ctx: Context<FreezeTokenAccount>) -> Result<()> {
    require!(
        !ctx.accounts.token_config.freeze_authority_revoked,
        TokenFactoryError::FreezeAuthorityRevoked
    );

    anchor_spl::token_2022::freeze_account(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token_2022::FreezeAccount {
                account:   ctx.accounts.target_account.to_account_info(),
                mint:      ctx.accounts.mint.to_account_info(),
                authority: ctx.accounts.creator.to_account_info(),
            },
        ),
    )?;

    msg!("Account frozen: {}", ctx.accounts.target_account.key());
    Ok(())
}

pub fn thaw_handler(ctx: Context<ThawTokenAccount>) -> Result<()> {
    require!(
        !ctx.accounts.token_config.freeze_authority_revoked,
        TokenFactoryError::FreezeAuthorityRevoked
    );

    anchor_spl::token_2022::thaw_account(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token_2022::ThawAccount {
                account:   ctx.accounts.target_account.to_account_info(),
                mint:      ctx.accounts.mint.to_account_info(),
                authority: ctx.accounts.creator.to_account_info(),
            },
        ),
    )?;

    msg!("Account thawed: {}", ctx.accounts.target_account.key());
    Ok(())
}
