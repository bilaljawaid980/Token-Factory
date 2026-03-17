use anchor_lang::prelude::*;

#[error_code]
pub enum TokenFactoryError {
    #[msg("Name exceeds maximum length of 32 characters")]
    NameTooLong,

    #[msg("Symbol exceeds maximum length of 10 characters")]
    SymbolTooLong,

    #[msg("URI exceeds maximum length of 200 characters")]
    UriTooLong,

    #[msg("Social link exceeds maximum length of 100 characters")]
    SocialTooLong,

    #[msg("Decimals must be between 0 and 9")]
    InvalidDecimals,

    #[msg("Initial supply must be greater than zero")]
    ZeroSupply,

    #[msg("Supply would exceed the maximum cap")]
    SupplyCapExceeded,

    #[msg("Mint authority has been permanently revoked")]
    MintAuthorityRevoked,

    #[msg("Freeze authority has been permanently revoked")]
    FreezeAuthorityRevoked,

    #[msg("Update authority has been permanently revoked")]
    UpdateAuthorityRevoked,

    #[msg("Only the token creator can perform this action")]
    UnauthorizedCreator,

    #[msg("Platform fee was not paid correctly")]
    PlatformFeeNotPaid,

    #[msg("Transfer fee basis points cannot exceed 10000")]
    InvalidTransferFee,

    #[msg("Max supply cap cannot be less than initial supply")]
    InvalidSupplyCap,
}