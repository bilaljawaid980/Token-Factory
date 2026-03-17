use anchor_lang::{prelude::*, system_program};
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_2022_extensions::{
    metadata_pointer::{metadata_pointer_initialize, MetadataPointerInitialize},
    token_metadata::{token_metadata_initialize, token_metadata_update_field, TokenMetadataInitialize, TokenMetadataUpdateField},
    transfer_fee::{transfer_fee_initialize, TransferFeeInitialize},
};
use spl_token_2022::{extension::ExtensionType, state::Mint as MintState};
use spl_token_metadata_interface::state::{Field, TokenMetadata};
use crate::{constants::*, errors::TokenFactoryError, state::TokenConfig};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeTokenParams {
    pub name:             String,
    pub symbol:           String,
    pub uri:              String,
    pub decimals:         u8,
    pub initial_supply:   u64,
    pub max_supply:       u64,
    pub transfer_fee_bps: u16,
    pub transfer_fee_max: u64,
    pub website:          String,
    pub twitter:          String,
    pub telegram:         String,
    pub discord:          String,
}

#[derive(Accounts)]
#[instruction(params: InitializeTokenParams)]
pub struct InitializeToken<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    /// CHECK: validated by address constraint
    #[account(
        mut,
        address = PLATFORM_FEE_WALLET.parse::<Pubkey>().unwrap()
    )]
    pub fee_wallet: AccountInfo<'info>,

    /// CHECK: fresh mint keypair, created manually via CPI
    #[account(mut)]
    pub mint: Signer<'info>,

    #[account(
        init,
        payer = creator,
        space = TokenConfig::LEN,
        seeds = [TOKEN_CONFIG_SEED, mint.key().as_ref()],
        bump,
    )]
    pub token_config: Account<'info, TokenConfig>,

    pub token_program:  Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    pub rent:           Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<InitializeToken>, params: InitializeTokenParams) -> Result<()> {
    // 1. Validate params
    require!(params.name.len() <= MAX_NAME_LENGTH, TokenFactoryError::NameTooLong);
    require!(params.symbol.len() <= MAX_SYMBOL_LENGTH, TokenFactoryError::SymbolTooLong);
    require!(params.uri.len() <= MAX_URI_LENGTH, TokenFactoryError::UriTooLong);
    require!(params.decimals <= 9, TokenFactoryError::InvalidDecimals);
    require!(params.initial_supply > 0, TokenFactoryError::ZeroSupply);
    require!(params.transfer_fee_bps <= 10_000, TokenFactoryError::InvalidTransferFee);
    require!(
        params.max_supply == 0 || params.max_supply >= params.initial_supply,
        TokenFactoryError::InvalidSupplyCap
    );

    // 2. Collect platform fee
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.creator.to_account_info(),
                to:   ctx.accounts.fee_wallet.to_account_info(),
            },
        ),
        PLATFORM_FEE_LAMPORTS,
    )?;

    // 3. Calculate space for mint + extensions only
    let extension_types = vec![
        ExtensionType::MetadataPointer,
        ExtensionType::TransferFeeConfig,
    ];
    let mint_size = ExtensionType::try_calculate_account_len::<MintState>(&extension_types)
        .map_err(|_| error!(TokenFactoryError::NameTooLong))?;

    // 4. Calculate extra space needed for metadata and pre-fund it
    let metadata = TokenMetadata {
        name:   params.name.clone(),
        symbol: params.symbol.clone(),
        uri:    params.uri.clone(),
        additional_metadata: vec![
            ("website".to_string(),  params.website.clone()),
            ("twitter".to_string(),  params.twitter.clone()),
            ("telegram".to_string(), params.telegram.clone()),
            ("discord".to_string(),  params.discord.clone()),
        ],
        ..Default::default()
    };
    let metadata_size = metadata.tlv_size_of()
        .map_err(|_| error!(TokenFactoryError::NameTooLong))?;
    let total_size = mint_size + 4 + usize::from(metadata_size);

    // 5. Allocate mint account with full size (mint + metadata space)
    let lamports = ctx.accounts.rent.minimum_balance(total_size);
    system_program::create_account(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::CreateAccount {
                from: ctx.accounts.creator.to_account_info(),
                to:   ctx.accounts.mint.to_account_info(),
            },
        ),
        lamports,
        mint_size as u64,   // allocate only mint size initially
        &spl_token_2022::ID,
    )?;

    // 6. Top up lamports for metadata reallocation
    let current_lamports = ctx.accounts.mint.lamports();
    if lamports > current_lamports {
        let diff = lamports - current_lamports;
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.creator.to_account_info(),
                    to:   ctx.accounts.mint.to_account_info(),
                },
            ),
            diff,
        )?;
    }

    // 7. Init MetadataPointer (before InitializeMint2)
    let mint_key = ctx.accounts.mint.key();
    metadata_pointer_initialize(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MetadataPointerInitialize {
                token_program_id: ctx.accounts.token_program.to_account_info(),
                mint:             ctx.accounts.mint.to_account_info(),
            },
        ),
        Some(ctx.accounts.creator.key()),
        Some(mint_key),
    )?;

    // 8. Init TransferFeeConfig (before InitializeMint2)
    transfer_fee_initialize(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferFeeInitialize {
                token_program_id: ctx.accounts.token_program.to_account_info(),
                mint:             ctx.accounts.mint.to_account_info(),
            },
        ),
        Some(&ctx.accounts.creator.key()),
        Some(&ctx.accounts.creator.key()),
        params.transfer_fee_bps,
        params.transfer_fee_max,
    )?;

    // 9. InitializeMint2 (after all extensions)
    anchor_spl::token_2022::initialize_mint2(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token_2022::InitializeMint2 {
                mint: ctx.accounts.mint.to_account_info(),
            },
        ),
        params.decimals,
        &ctx.accounts.creator.key(),
        Some(&ctx.accounts.creator.key()),
    )?;

    // 10. Init on-chain metadata (after InitializeMint2)
    token_metadata_initialize(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TokenMetadataInitialize {
                program_id:       ctx.accounts.token_program.to_account_info(),
                metadata:         ctx.accounts.mint.to_account_info(),
                update_authority: ctx.accounts.creator.to_account_info(),
                mint_authority:   ctx.accounts.creator.to_account_info(),
                mint:             ctx.accounts.mint.to_account_info(),
            },
        ),
        params.name.clone(),
        params.symbol.clone(),
        params.uri.clone(),
    )?;

    // 11. Write social links
    for (key, value) in [
        ("website",  params.website.clone()),
        ("twitter",  params.twitter.clone()),
        ("telegram", params.telegram.clone()),
        ("discord",  params.discord.clone()),
    ] {
        token_metadata_update_field(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TokenMetadataUpdateField {
                    program_id:       ctx.accounts.token_program.to_account_info(),
                    metadata:         ctx.accounts.mint.to_account_info(),
                    update_authority: ctx.accounts.creator.to_account_info(),
                },
            ),
            Field::Key(key.to_string()),
            value,
        )?;
    }

    // 12. Save TokenConfig PDA
    let config = &mut ctx.accounts.token_config;
    config.creator                  = ctx.accounts.creator.key();
    config.mint                     = ctx.accounts.mint.key();
    config.decimals                 = params.decimals;
    config.initial_supply           = params.initial_supply;
    config.max_supply               = params.max_supply;
    config.current_supply           = 0;
    config.transfer_fee_bps         = params.transfer_fee_bps;
    config.transfer_fee_max         = params.transfer_fee_max;
    config.mint_authority_revoked   = false;
    config.freeze_authority_revoked = false;
    config.update_authority_revoked = false;
    config.bump                     = ctx.bumps.token_config;

    msg!(
        "Token created: {} ({}) | supply: {} | fee: {}bps",
        params.name, params.symbol, params.initial_supply, params.transfer_fee_bps
    );

    Ok(())
}