use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_2022::Token2022};
use crate::{constants::TOKEN_CONFIG_SEED, errors::TokenFactoryError, state::TokenConfig};

#[derive(Accounts)]
pub struct MintInitialSupply<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    /// CHECK: Token-2022 mint, validated by token_config seeds
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [TOKEN_CONFIG_SEED, mint.key().as_ref()],
        bump = token_config.bump,
        has_one = creator @ TokenFactoryError::UnauthorizedCreator,
    )]
    pub token_config: Account<'info, TokenConfig>,

    #[account(
        init_if_needed,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = creator,
        associated_token::token_program = token_program,
    )]
    pub creator_ata: InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>,

    pub token_program:            Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program:           Program<'info, System>,
}

pub fn handler(ctx: Context<MintInitialSupply>) -> Result<()> {
    let config = &mut ctx.accounts.token_config;

    require!(config.current_supply == 0, TokenFactoryError::ZeroSupply);
    require!(!config.mint_authority_revoked, TokenFactoryError::MintAuthorityRevoked);

    let supply_with_decimals = config.initial_supply
        .checked_mul(10u64.pow(config.decimals as u32))
        .ok_or(TokenFactoryError::SupplyCapExceeded)?;

    anchor_spl::token_2022::mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token_2022::MintTo {
                mint:      ctx.accounts.mint.to_account_info(),
                to:        ctx.accounts.creator_ata.to_account_info(),
                authority: ctx.accounts.creator.to_account_info(),
            },
        ),
        supply_with_decimals,
    )?;

    config.current_supply = config.initial_supply;

    msg!("Initial supply minted: {} tokens", config.initial_supply);
    Ok(())
}
