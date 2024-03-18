use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, Transfer};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

declare_id!("7GZSUfS2MkyV1gCMbmWtP4K4QEJgFHdassxB1ZXZ2n9N");

#[program]
pub mod stking_dapp {
    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Instruction Initialize");

        let pool_info = &mut ctx.accounts.pool_info;

        if !pool_info.is_initialized {
            pool_info.is_initialized = true;
            pool_info.admin = ctx.accounts.admin.key();
            pool_info.token = ctx.accounts.staking_token.key();
            pool_info.amount = 0;
        }
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64, lockedays: u64) -> Result<()> {
        msg!("Instruction Stake");

        let user_info = &mut ctx.accounts.user_info;
        let pool_info = &mut ctx.accounts.pool_info;

        if !user_info.is_initialized {
            user_info.is_initialized = true;
            user_info.amount = 0;
            user_info.reward = 0;
            user_info.deposit_slot = 1;
        }

        let clock = Clock::get()?;
        let current_timestamp = clock.slot as u64;
        let timestamp = current_timestamp.to_string();
        if current_timestamp - user_info.deposit_slot < user_info.locked_days * 24 * 60 * 60
            || user_info.deposit_slot == 0
        {
            ();
        }

        if user_info.reward > 0 {
            let reward = user_info.reward;
            let cpi_accounts = MintTo {
                mint: ctx.accounts.staking_token.to_account_info(),
                to: ctx.accounts.user_staking_wallet.to_account_info(),
                authority: ctx.accounts.admin_staking_wallet.to_account_info(),
            };

            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

            token::mint_to(cpi_ctx, reward)?;
        }

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_staking_wallet.to_account_info(),
            to: ctx.accounts.admin_staking_wallet.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::transfer(cpi_ctx, amount)?;

        user_info.amount += amount;
        user_info.deposit_slot = clock.slot as u64;
        user_info.locked_days = lockedays;
        user_info.reward = user_info.amount * user_info.locked_days / 10;
        pool_info.amount += amount;
        Ok(msg!(&timestamp))
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    
    #[account(init_if_needed, seeds=[b"pool", admin.key().as_ref()], bump, payer = admin, space = std::mem::size_of::<PoolInfo>() + 8 )]
    pub pool_info: Account<'info, PoolInfo>,

    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub staking_token: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut, seeds=[b"pool", admin.key().as_ref()], bump )]
    pub pool_info: Account<'info, PoolInfo>,

    #[account(init_if_needed, seeds=[b"user", user.key().as_ref()], bump, payer = user, space= std::mem::size_of::<UserInfo>() + 8 )]
    pub user_info: Account<'info, UserInfo>,

    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub user_staking_wallet: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub admin_staking_wallet: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub staking_token: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct UserInfo {
    pub is_initialized: bool,
    pub amount: u64,
    pub deposit_slot: u64,
    pub locked_days: u64,
    pub reward: u64,
}

#[account]
pub struct PoolInfo {
    pub is_initialized: bool,
    pub admin: Pubkey,
    pub token: Pubkey,
    pub amount: u64,
}

impl UserInfo {
    pub const LEN: usize = 1 + 8 + 8 + 8 + 8;
}

impl PoolInfo {
    pub const LEN: usize = 1 + 32 + 32 + 8;
}
