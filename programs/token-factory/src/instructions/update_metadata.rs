use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_2022_extensions::token_metadata::{
    token_metadata_update_field, TokenMetadataUpdateField,
};
use spl_token_metadata_interface::state::Field;
use crate::{errors::TokenFactoryError, state::TokenConfig, constants::TOKEN_CONFIG_SEED};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateMetadataParams {
    pub name:     Option<String>,
    pub symbol:   Option<String>,
    pub uri:      Option<String>,
    pub website:  Option<String>,
    pub twitter:  Option<String>,
    pub telegram: Option<String>,
    pub discord:  Option<String>,
}

#[derive(Accounts)]
pub struct UpdateMetadata<'info> {
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

pub fn handler(ctx: Context<UpdateMetadata>, params: UpdateMetadataParams) -> Result<()> {
    require!(
        !ctx.accounts.token_config.update_authority_revoked,
        TokenFactoryError::UpdateAuthorityRevoked
    );

    let fields: Vec<(Field, String)> = vec![
        params.name.map(|v|     (Field::Name, v)),
        params.symbol.map(|v|   (Field::Symbol, v)),
        params.uri.map(|v|      (Field::Uri, v)),
        params.website.map(|v|  (Field::Key("website".to_string()), v)),
        params.twitter.map(|v|  (Field::Key("twitter".to_string()), v)),
        params.telegram.map(|v| (Field::Key("telegram".to_string()), v)),
        params.discord.map(|v|  (Field::Key("discord".to_string()), v)),
    ]
    .into_iter()
    .flatten()
    .collect();

    for (field, value) in fields {
        token_metadata_update_field(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TokenMetadataUpdateField {
                    program_id:       ctx.accounts.token_program.to_account_info(),
                    metadata:         ctx.accounts.mint.to_account_info(),
                    update_authority: ctx.accounts.creator.to_account_info(),
                },
            ),
            field,
            value,
        )?;
    }

    msg!("Metadata updated");
    Ok(())
}
