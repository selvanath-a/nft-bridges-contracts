use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey;
use solana_program::program::invoke;

use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_master_edition_v3, create_metadata_accounts_v3,
        set_and_verify_sized_collection_item, sign_metadata, CreateMasterEditionV3,
        CreateMetadataAccountsV3, Metadata, SetAndVerifySizedCollectionItem, SignMetadata,
    },
    token::{mint_to, Mint, MintTo, Token, TokenAccount, Transfer},
};
use mpl_token_metadata::accounts::{MasterEdition, Metadata as MetadataAccount};
use mpl_token_metadata::types::{CollectionDetails, Creator, DataV2};

declare_id!("ETWdEcjv3mCb9QzS9Kb6vqv7fi8c3hpLW7Jcsrz2hmEE");

#[constant]
pub const NFT_INFO: &str = "nft_info";
pub const BRIDGE: &str = "bridge";

#[program]
pub mod anchor_nft_collection {
    use super::*;

    pub fn initialize_bridge(_ctx: Context<InitializeBridge>) -> Result<()> {
        Ok(())
    }

    pub fn initialize_and_lock_nft_fee(
        ctx: Context<InitializeAndLockNftFee>,
        origin_chain: String,
        origin_contract_address: String,
        nft_id: u64,
        amount: u64,
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

        let nft_info_account = &mut ctx.accounts.nft_info_account;
        nft_info_account.mint_address = ctx.accounts.mint_of_token_being_sent.key();

        msg!("NFT ID: {}", nft_id);
        msg!("Collection ID: {}", coll_id);
        msg!("Source Address: {}", src_address);
        msg!("Destination Chain: {}", dst_chain);
        msg!("Destination Address: {}", dst_address);

        Ok(())
    }

    pub fn unlock_nft(
        ctx: Context<UnlockNft>,
        origin_chain: String,
        origin_contract_address: String,
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
        let seeds = &[BRIDGE.as_bytes().as_ref(), &[bump]];
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


    pub fn store_nft_info_in_bridge(
        ctx: Context<StoreNftInfoInBridge>,
        origin_chain: String,            // origin_chain passed from client
        origin_contract_address: String, // origin_contract_address passed from client
        mint_address: Pubkey,
        nft_id: u64,
    ) -> Result<()> {
        let nft_info_account = &mut ctx.accounts.nft_info_account;

        nft_info_account.mint_address = mint_address;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeBridge<'info> {
    // Derived PDAs
    #[account(
        init_if_needed,
        payer = signer,
        seeds=[BRIDGE.as_bytes()],
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
#[instruction(
        origin_chain: String,
        origin_contract_address: String,
        nft_id: u64,
        amount: u64,
        coll_id: String,
        src_address: String,
        dst_chain: String,
        dst_address: String,)]
pub struct InitializeAndLockNftFee<'info> {
    // Derived PDAs and accounts for initialization
    #[account(
        mut,
        seeds=[BRIDGE.as_bytes()],
        bump
    )]
    pub bridge_pda: AccountInfo<'info>, // `mut` is needed because the PDA will be initialized

    pub mint_of_token_being_sent: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = signer,
        seeds = [&nft_id.to_le_bytes(), origin_chain.as_bytes(), origin_contract_address.as_bytes()],
        token::mint = mint_of_token_being_sent,
        token::authority = bridge_pda,
        bump
    )]
    pub nft_token_account: Account<'info, TokenAccount>, // `mut` is needed because the account will be initialized

    #[account(
        init_if_needed,
        seeds = [NFT_INFO.as_bytes(),&nft_id.to_le_bytes(), origin_chain.as_bytes(), origin_contract_address.as_bytes()],
        bump,
        payer = signer,
        space = 8 + std::mem::size_of::<NftInfoInBridge>(),
    )]
    pub nft_info_account: Box<Account<'info, NftInfoInBridge>>,

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
#[instruction(origin_chain: String,
        origin_contract_address: String,
        nft_id: u64,
        coll_id: String,
        src_chain: String,
        src_address: String,
        dst_address: String,
        bridge_txid: String,)]
pub struct UnlockNft<'info> {
    // Derived PDAs
    #[account(
        mut,
        seeds=[BRIDGE.as_bytes()],
        bump
    )]
    bridge_pda: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [&nft_id.to_le_bytes(), origin_chain.as_bytes(), origin_contract_address.as_bytes()],
        bump,
        token::mint=mint_of_token_being_sent,
        token::authority=bridge_pda,
    )]
    nft_token_account: Account<'info, TokenAccount>,

    mint_of_token_being_sent: Account<'info, Mint>,

    #[account(mut)]
    pub receiver: AccountInfo<'info>,

    #[account(init_if_needed , payer = signer ,
    associated_token::mint = mint_of_token_being_sent,
    associated_token::authority = receiver)]
    receiver_token_account: Account<'info, TokenAccount>,

    //Change it to the address you want
    #[account(mut,address=pubkey!("7QHySLfCkeSBUGxtqM3WdeKv9X4YUXMeTn3gr5fRPCGU"))]
    signer: Signer<'info>,
    system_program: Program<'info, System>,
    token_program: Program<'info, Token>,
    associated_token_program: Program<'info, AssociatedToken>,
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(origin_chain: String , origin_contract_address:String ,mint_address: Pubkey, nft_id: u64)]
pub struct StoreNftInfoInBridge<'info> {
    #[account(
        init_if_needed,
        // seeds = [NFT_INFO.as_bytes(),nft_id.as_bytes(), origin_chain.as_bytes(), origin_contract_address.as_bytes()],
        seeds = [NFT_INFO.as_bytes(),&nft_id.to_le_bytes(), origin_chain.as_bytes(), origin_contract_address.as_bytes()],
        bump,
        payer = signer,
        space = 8 +std::mem::size_of::<NftInfoInBridge>(),
    )]
    pub nft_info_account: Box<Account<'info, NftInfoInBridge>>,

    #[account(mut,address=pubkey!("7QHySLfCkeSBUGxtqM3WdeKv9X4YUXMeTn3gr5fRPCGU"))]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct NftInfoInBridge {
    pub mint_address: Pubkey,
}
