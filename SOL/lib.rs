use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};
use solana_program::program::invoke;

declare_id!("7dX8QYJfiMv62X2MtRxE2MTacBHKvHJpzVw71yiAbCtn");

#[program]
mod nft_bridge {
    use super::*;

    pub fn initialize_bridge(_ctx: Context<InitializeBridge>) -> Result<()> {
        Ok(())
    }
    pub fn initialize_and_lock_nft(
        ctx: Context<InitializeAndLockNft>,
        nft_id: u64,
        coll_id: String,
        src_address: String,
        dst_chain: String,
        dst_address: String,
    ) -> Result<()> {
        // Initialization logic (creating PDAs, etc.)
        msg!("Initialization successful!");
        // Transfer tokens from sender's token account to the bridge's NFT token account
        let transfer_instruction = Transfer {
            from: ctx.accounts.sender_token_account.to_account_info(),
            to: ctx.accounts.nft_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        );

        // Perform the transfer
        anchor_spl::token::transfer(cpi_ctx, 1)?;

        msg!("NFT ID: {}", nft_id);
        msg!("Collection ID: {}", coll_id);
        msg!("Source Address: {}", src_address);
        msg!("Destination Chain: {}", dst_chain);
        msg!("Destination Address: {}", dst_address);

        Ok(())
    }

    pub fn initialize_and_lock_nft_fee(
        ctx: Context<InitializeAndLockNftFee>,
        amount: u64,
        nft_id: u64,
        coll_id: String,
        src_address: String,
        dst_chain: String,
        dst_address: String,
    ) -> Result<()> {
        // Initialization logic (creating PDAs, etc.)
        msg!("Initialization successful!");

        // Token transfer logic (moving tokens into the bridge)
        msg!("Token amount transfer in: 1 ");

        // Transfer tokens from sender's token account to the bridge's NFT token account
        let transfer_instruction = Transfer {
            from: ctx.accounts.sender_token_account.to_account_info(),
            to: ctx.accounts.nft_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        );

        // Perform the transfer
        anchor_spl::token::transfer(cpi_ctx, 1)?;

        let sol_transfer = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.from_pubkey.key,
            &ctx.accounts.to_pubkey.key,
            amount,
        );
        invoke(
            &sol_transfer,
            &[
                ctx.accounts.from_pubkey.clone(),
                ctx.accounts.to_pubkey.clone(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        msg!("NFT ID: {}", nft_id);
        msg!("Collection ID: {}", coll_id);
        msg!("Source Address: {}", src_address);
        msg!("Destination Chain: {}", dst_chain);
        msg!("Destination Address: {}", dst_address);

        Ok(())
    }

    pub fn unlock_nft(
        ctx: Context<UnlockNft>,
        nft_id: u64,
        coll_id: String,
        src_chain: String,
        src_address: String,
        dst_address: String,
        bridge_txid: String,
    ) -> Result<()> {
        msg!("Token amount transfer out: 1 ");

        // Below is the actual instruction that we are going to send to the Token program.
        let transfer_instruction = Transfer {
            from: ctx.accounts.nft_token_account.to_account_info(),
            to: ctx.accounts.receiver_token_account.to_account_info(),
            authority: ctx.accounts.bridge_pda.to_account_info(),
        };

        let bump = ctx.bumps.bridge_pda;
        let seeds = &[b"bridge".as_ref(), &[bump]];
        let signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            signer,
        );

        anchor_spl::token::transfer(cpi_ctx, 1)?;

        msg!("NFT ID: {}", nft_id);
        msg!("Collection ID: {}", coll_id);
        msg!("Source Chain: {}", src_chain);
        msg!("Source Address: {}", src_address);
        msg!("Destination Address: {}", dst_address);
        msg!("Bridge TxId: {}", bridge_txid);

        Ok(())
    }

}

#[derive(Accounts)]
pub struct InitializeBridge<'info> {
    // Derived PDAs
    #[account(
        init_if_needed,
        payer = signer,
        seeds=[b"bridge"],
        bump,
        space = 8 + 8
    )]
    bridge_pda: AccountInfo<'info>,

    #[account(mut,address=pubkey!("7QHySLfCkeSBUGxtqM3WdeKv9X4YUXMeTn3gr5fRPCGU"))]
    signer: Signer<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitializeAndLockNft<'info> {
    // Derived PDAs and accounts for initialization
    #[account(
        mut,
        seeds=[b"bridge"],
        bump
    )]
    pub bridge_pda: AccountInfo<'info>, // `mut` is needed because the PDA will be initialized

    pub mint_of_token_being_sent: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = signer,
        seeds = [b"nft", mint_of_token_being_sent.key().as_ref()],
        token::mint = mint_of_token_being_sent,
        token::authority = bridge_pda,
        bump
    )]
    pub nft_token_account: Account<'info, TokenAccount>, // `mut` is needed because the account will be initialized

    #[account(mut)]
    pub sender_token_account: Account<'info, TokenAccount>, // `mut` is needed because the account will be used for transfer

    #[account(mut)]
    pub signer: Signer<'info>, // `mut` is needed because the signer is part of the transaction

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitializeAndLockNftFee<'info> {
    // Derived PDAs and accounts for initialization
    #[account(
        mut,
        seeds=[b"bridge"],
        bump
    )]
    pub bridge_pda: AccountInfo<'info>, // `mut` is needed because the PDA will be initialized

    pub mint_of_token_being_sent: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = signer,
        seeds = [b"nft", mint_of_token_being_sent.key().as_ref()],
        token::mint = mint_of_token_being_sent,
        token::authority = bridge_pda,
        bump
    )]
    pub nft_token_account: Account<'info, TokenAccount>, // `mut` is needed because the account will be initialized

    #[account(mut)]
    pub sender_token_account: Account<'info, TokenAccount>, // `mut` is needed because the account will be used for transfer

    #[account(mut)]
    pub to_pubkey: AccountInfo<'info>,

    #[account(mut)]
    pub from_pubkey: AccountInfo<'info>,

    #[account(mut)]
    pub signer: Signer<'info>, // `mut` is needed because the signer is part of the transaction

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UnlockNft<'info> {
    // Derived PDAs
    #[account(
        mut,
        seeds=[b"bridge"],
        bump
    )]
    bridge_pda: AccountInfo<'info>,

    #[account(
        mut,
        seeds=[b"nft", mint_of_token_being_sent.key().as_ref()],
        bump,
        token::mint=mint_of_token_being_sent,
        token::authority=bridge_pda,
    )]
    nft_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    receiver_token_account: Account<'info, TokenAccount>,

    mint_of_token_being_sent: Account<'info, Mint>,

    //Change it to the address you want
    #[account(mut,address=pubkey!("7QHySLfCkeSBUGxtqM3WdeKv9X4YUXMeTn3gr5fRPCGU"))]
    signer: Signer<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
}
