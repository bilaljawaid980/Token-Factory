use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct TokenConfig {
    // Creator wallet — authority for all gated instructions
    pub creator: Pubkey,               // 32

    // The Token-2022 mint this config belongs to
    pub mint: Pubkey,                  // 32

    // Token parameters
    pub decimals: u8,                  // 1
    pub initial_supply: u64,           // 8
    pub max_supply: u64,               // 8  (0 = uncapped)
    pub current_supply: u64,           // 8

    // Transfer fee config (basis points, e.g. 100 = 1%)
    pub transfer_fee_bps: u16,         // 2
    pub transfer_fee_max: u64,         // 8

    // Permanent revocation flags — once true, cannot be undone
    pub mint_authority_revoked: bool,  // 1
    pub freeze_authority_revoked: bool,// 1
    pub update_authority_revoked: bool,// 1

    // PDA bump
    pub bump: u8,                      // 1
}

impl TokenConfig {
    // 8 (discriminator) + all fields above
    pub const LEN: usize = 8 + 32 + 32 + 1 + 8 + 8 + 8 + 2 + 8 + 1 + 1 + 1 + 1;

    pub fn is_under_cap(&self, additional: u64) -> bool {
        if self.max_supply == 0 {
            return true; // uncapped
        }
        self.current_supply
            .checked_add(additional)
            .map(|total| total <= self.max_supply)
            .unwrap_or(false)
    }
}