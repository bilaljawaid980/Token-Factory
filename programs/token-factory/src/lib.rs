use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod state;
pub mod instructions;

use instructions::initialize_token::*;
use instructions::mint_initial_supply::*;
use instructions::revoke_authority::*;
use instructions::mint_additional::*;
use instructions::update_metadata::*;
use instructions::freeze_account::*;
use instructions::harvest_fees::*;

declare_id!("9UgZ4TDiiaMgu6ybgBne112k6xnb8AfMWwrr1e4hHUZZ");

#[program]
pub mod token_factory {
    use super::*;

    pub fn initialize_token(
        ctx: Context<InitializeToken>,
        params: InitializeTokenParams,
    ) -> Result<()> {
        instructions::initialize_token::handler(ctx, params)
    }

    pub fn mint_initial_supply(
        ctx: Context<MintInitialSupply>,
    ) -> Result<()> {
        instructions::mint_initial_supply::handler(ctx)
    }

    pub fn revoke_authority(
        ctx: Context<RevokeAuthority>,
        params: RevokeAuthorityParams,
    ) -> Result<()> {
        instructions::revoke_authority::handler(ctx, params)
    }

    pub fn mint_additional(
        ctx: Context<MintAdditional>,
        amount: u64,
    ) -> Result<()> {
        instructions::mint_additional::handler(ctx, amount)
    }

    pub fn update_metadata(
        ctx: Context<UpdateMetadata>,
        params: UpdateMetadataParams,
    ) -> Result<()> {
        instructions::update_metadata::handler(ctx, params)
    }

    pub fn freeze_token_account(
        ctx: Context<FreezeTokenAccount>,
    ) -> Result<()> {
        instructions::freeze_account::handler(ctx)
    }

    pub fn thaw_token_account(
        ctx: Context<ThawTokenAccount>,
    ) -> Result<()> {
        instructions::freeze_account::thaw_handler(ctx)
    }

    pub fn harvest_fees(
        ctx: Context<HarvestFees>,
    ) -> Result<()> {
        instructions::harvest_fees::handler(ctx)
    }
}
