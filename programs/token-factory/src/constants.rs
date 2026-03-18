

// Seeds
pub const TOKEN_CONFIG_SEED: &[u8] = b"token_config";
pub const MINT_AUTHORITY_SEED: &[u8] = b"mint_authority";

// Platform fee (0.1 SOL in lamports)
pub const PLATFORM_FEE_LAMPORTS: u64 = 100_000_000;

// Platform fee wallet — replace with your actual devnet wallet pubkey
pub const PLATFORM_FEE_WALLET: &str = "6RyiHzXdvrucGp332NML2xNrzTzBGqKGHjUdWzdsQdtW";

// Token-2022 extension space constants
pub const MAX_NAME_LENGTH: usize = 32;
pub const MAX_SYMBOL_LENGTH: usize = 10;
pub const MAX_URI_LENGTH: usize = 200;
pub const MAX_SOCIAL_LENGTH: usize = 100;