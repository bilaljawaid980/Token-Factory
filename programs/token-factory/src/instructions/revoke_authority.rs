use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use crate::{errors::TokenFactoryError, state::TokenConfig, constants::TOKEN_CONFIG_SEED};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RevokeAuthorityParams {
    pub revoke_mint:   bool,
    pub revoke_freeze: bool,
    pub revoke_update: bool,
}

#[derive(Accounts)]
pub struct RevokeAuthority<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    /// CHECK: Token-2022 mint account, validated by token_config PDA seeds
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [TOKEN_CONFIG_SEED, mint.key().as_ref()],
        bump = token_config.bump,
        has_one = creator @ TokenFactoryError::UnauthorizedCreator,
    )]
    pub token_config: Account<'info, TokenConfig>,

    pub token_program:  Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RevokeAuthority>, params: RevokeAuthorityParams) -> Result<()> {
    let config = &mut ctx.accounts.token_config;

    if params.revoke_mint {
        require!(!config.mint_authority_revoked, TokenFactoryError::MintAuthorityRevoked);
        anchor_spl::token_2022::set_authority(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token_2022::SetAuthority {
                    account_or_mint:   ctx.accounts.mint.to_account_info(),
                    current_authority: ctx.accounts.creator.to_account_info(),
                },
            ),
            anchor_spl::token_2022::spl_token_2022::instruction::AuthorityType::MintTokens,
            None,
        )?;
        config.mint_authority_revoked = true;
        msg!("Mint authority permanently revoked");
    }

    if params.revoke_freeze {
        require!(!config.freeze_authority_revoked, TokenFactoryError::FreezeAuthorityRevoked);
        anchor_spl::token_2022::set_authority(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token_2022::SetAuthority {
                    account_or_mint:   ctx.accounts.mint.to_account_info(),
                    current_authority: ctx.accounts.creator.to_account_info(),
                },
            ),
            anchor_spl::token_2022::spl_token_2022::instruction::AuthorityType::FreezeAccount,
            None,
        )?;
        config.freeze_authority_revoked = true;
        msg!("Freeze authority permanently revoked");
    }

    if params.revoke_update {
        require!(!config.update_authority_revoked, TokenFactoryError::UpdateAuthorityRevoked);
        config.update_authority_revoked = true;
        msg!("Update authority permanently revoked");
    }

    Ok(())
}
