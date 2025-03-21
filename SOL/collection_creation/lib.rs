use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey;
use anchor_lang::solana_program::{hash::hash, system_instruction, sysvar::SysvarId};

use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_master_edition_v3, create_metadata_accounts_v3,
        set_and_verify_sized_collection_item, sign_metadata, CreateMasterEditionV3,
        CreateMetadataAccountsV3, Metadata, SetAndVerifySizedCollectionItem, SignMetadata,
    },
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};
use mpl_token_metadata::accounts::{MasterEdition, Metadata as MetadataAccount};
use mpl_token_metadata::types::{CollectionDetails, Creator, DataV2};

declare_id!("AizEzdXgSms3KjkNEsBycmsmJD7LQa2wChwKHaKVXoix");

#[constant]
pub const COLLECTION: &str = "Collection";
pub const BRIDGE: &str = "Bridge";
pub const COLLECTION_INFO: &str = "Collection_Info";

#[program]
pub mod collection_creator {
    use super::*;

    pub fn store_collection_info(
        ctx: Context<StoreCollectionInfo>,
        origin_chain: String,            // origin_chain passed from client
        origin_contract_address: String, // origin_contract_address passed from client
    ) -> Result<()> {
        let collection_info_account = &mut ctx.accounts.collection_info_account;

        // Store the provided origin_chain and origin_contract_address in the account
        collection_info_account.origin_chain = origin_chain;
        collection_info_account.origin_contract_address = origin_contract_address;

        Ok(())
    }

    pub fn create_collection_nft(
        ctx: Context<CreateCollectionNft>,
        uri: String,
        name: String,
        symbol: String,
        origin_chain: String,
        origin_contract_address: String,
    ) -> Result<()> {
        // PDA for signing

        let collection_info_account = &mut ctx.accounts.collection_info_account;

        if collection_info_account.origin_chain == "SOL" {
            msg!("Collection Already present in Solana !!")
        } else {
            let signer_seeds: &[&[&[u8]]] = &[&[
                origin_chain.as_bytes().as_ref(),
                origin_contract_address.as_bytes().as_ref(),
                &[ctx.bumps.collection_mint],
            ]];

            // mint collection nft
            mint_to(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    MintTo {
                        mint: ctx.accounts.collection_mint.to_account_info(),
                        to: ctx.accounts.token_account.to_account_info(),
                        authority: ctx.accounts.collection_mint.to_account_info(),
                    },
                    signer_seeds,
                ),
                1,
            )?;

            // create metadata account for collection nft
        }
        Ok(())
    }

    pub fn create_collection_nft2(
        ctx: Context<CreateCollectionNft>,
        uri: String,
        name: String,
        symbol: String,
        origin_chain: String,
        origin_contract_address: String,
    ) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            origin_chain.as_bytes().as_ref(),
            origin_contract_address.as_bytes().as_ref(),
            &[ctx.bumps.collection_mint],
        ]];

        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.metadata_account.to_account_info(),
                    mint: ctx.accounts.collection_mint.to_account_info(),
                    mint_authority: ctx.accounts.collection_mint.to_account_info(), // use pda mint address as mint authority
                    update_authority: ctx.accounts.collection_mint.to_account_info(), // use pda mint as update authority
                    payer: ctx.accounts.authority.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
            DataV2 {
                name: name,
                symbol: symbol,
                uri: uri,
                seller_fee_basis_points: 0,
                creators: Some(vec![Creator {
                    address: ctx.accounts.authority.key(),
                    verified: false,
                    share: 100,
                }]),
                collection: None,
                uses: None,
            },
            true,
            false,
            Some(CollectionDetails::V1 { size: 0 }), // set as collection nft
        )?;

        // create master edition account for collection nft
        create_master_edition_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMasterEditionV3 {
                    payer: ctx.accounts.authority.to_account_info(),
                    mint: ctx.accounts.collection_mint.to_account_info(),
                    edition: ctx.accounts.master_edition.to_account_info(),
                    mint_authority: ctx.accounts.collection_mint.to_account_info(),
                    update_authority: ctx.accounts.collection_mint.to_account_info(),
                    metadata: ctx.accounts.metadata_account.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
            Some(0),
        )?;

        // verify creator on metadata account
        sign_metadata(CpiContext::new(
            ctx.accounts.token_metadata_program.to_account_info(),
            SignMetadata {
                creator: ctx.accounts.authority.to_account_info(),
                metadata: ctx.accounts.metadata_account.to_account_info(),
            },
        ))?;
        Ok(())
    }

    pub fn create_nft_in_collection(
        ctx: Context<CreateNftInCollection>,
        uri: String,
        name: String,
        symbol: String,
        origin_chain: String,
        origin_contract_address: String,
    ) -> Result<()> {
        // let signer_seeds: &[&[&[u8]]] = &[&[SEED.as_bytes(), &[*ctx.bumps.collection_mint]]];

        let collection_info_account = &mut ctx.accounts.collection_info_account;

        if collection_info_account.origin_chain == "SOL" {
            msg!("Collection Already present in Solana !!")
        } else {
            let signer_seeds: &[&[&[u8]]] = &[&[
                origin_chain.as_bytes().as_ref(),
                origin_contract_address.as_bytes().as_ref(),
                &[ctx.bumps.collection_mint],
            ]];

            // mint nft in collection
            mint_to(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    MintTo {
                        mint: ctx.accounts.nft_mint.to_account_info(),
                        to: ctx.accounts.token_account.to_account_info(),
                        authority: ctx.accounts.collection_mint.to_account_info(),
                    },
                    signer_seeds,
                ),
                1,
            )?;

            create_metadata_accounts_v3(
                CpiContext::new_with_signer(
                    ctx.accounts.token_metadata_program.to_account_info(),
                    CreateMetadataAccountsV3 {
                        metadata: ctx.accounts.metadata_account.to_account_info(),
                        mint: ctx.accounts.nft_mint.to_account_info(),
                        mint_authority: ctx.accounts.collection_mint.to_account_info(),
                        update_authority: ctx.accounts.collection_mint.to_account_info(),
                        payer: ctx.accounts.signer.to_account_info(),
                        system_program: ctx.accounts.system_program.to_account_info(),
                        rent: ctx.accounts.rent.to_account_info(),
                    },
                    &signer_seeds,
                ),
                DataV2 {
                    name: name,
                    symbol: symbol,
                    uri: uri,
                    seller_fee_basis_points: 0,
                    creators: None,
                    collection: None,
                    uses: None,
                },
                true,
                true,
                None,
            )?;

            // create master edition account for nft in collection
            create_master_edition_v3(
                CpiContext::new_with_signer(
                    ctx.accounts.token_metadata_program.to_account_info(),
                    CreateMasterEditionV3 {
                        payer: ctx.accounts.signer.to_account_info(),
                        mint: ctx.accounts.nft_mint.to_account_info(),
                        edition: ctx.accounts.master_edition.to_account_info(),
                        mint_authority: ctx.accounts.collection_mint.to_account_info(),
                        update_authority: ctx.accounts.collection_mint.to_account_info(),
                        metadata: ctx.accounts.metadata_account.to_account_info(),
                        token_program: ctx.accounts.token_program.to_account_info(),
                        system_program: ctx.accounts.system_program.to_account_info(),
                        rent: ctx.accounts.rent.to_account_info(),
                    },
                    &signer_seeds,
                ),
                Some(0),
            )?;
        }

        Ok(())
    }

    pub fn verify_nft_in_collection(
        ctx: Context<VerifyNftInCollection>,
        origin_chain: String,
        origin_contract_address: String,
    ) -> Result<()> {
        let collection_info_account = &mut ctx.accounts.collection_info_account;

        if collection_info_account.origin_chain == "SOL" {
            msg!("Collection Already present in Solana !!")
        } else {
            let signer_seeds: &[&[&[u8]]] = &[&[
                origin_chain.as_bytes().as_ref(),
                origin_contract_address.as_bytes().as_ref(),
                &[ctx.bumps.collection_mint],
            ]];

            // verify nft as part of collection
            set_and_verify_sized_collection_item(
                CpiContext::new_with_signer(
                    ctx.accounts.token_metadata_program.to_account_info(),
                    SetAndVerifySizedCollectionItem {
                        metadata: ctx.accounts.metadata_account.to_account_info(),
                        collection_authority: ctx.accounts.collection_mint.to_account_info(),
                        payer: ctx.accounts.signer.to_account_info(),
                        update_authority: ctx.accounts.collection_mint.to_account_info(),
                        collection_mint: ctx.accounts.collection_mint.to_account_info(),
                        collection_metadata: ctx
                            .accounts
                            .collection_metadata_account
                            .to_account_info(),
                        collection_master_edition: ctx
                            .accounts
                            .collection_master_edition
                            .to_account_info(),
                    },
                    &signer_seeds,
                ),
                None,
            )?;
        }
        Ok(())
    }

}

#[derive(Accounts)]
#[instruction(
        uri: String,
        name: String,
        symbol: String,
        origin_chain: String,
        origin_contract_address:String)]
pub struct CreateCollectionNft<'info> {
    #[account(mut,address=pubkey!("7QHySLfCkeSBUGxtqM3WdeKv9X4YUXMeTn3gr5fRPCGU"))]
    pub authority: Signer<'info>,

    #[account(
        init_if_needed,
        payer = authority,
        seeds=[BRIDGE.as_bytes()],
        bump,
        space = 8 + 8
    )]
    bridge_pda: AccountInfo<'info>,

    #[account(
    init_if_needed,
    payer = authority,
    mint::decimals = 0,
    mint::authority = collection_mint,
    mint::freeze_authority = collection_mint,
    seeds = [origin_chain.as_bytes() , origin_contract_address.as_bytes()],
    bump,
    // seeds = [COLLECTION.as_bytes() , origin_chain.as_bytes() , origin_contract_address.as_bytes()],
    // seeds = [hash([COLLECTION , &origin_chain , &origin_contract_address].concat().as_bytes()).as_ref()],
    )]
    pub collection_mint: Account<'info, Mint>,

    /// CHECK:
    #[account(
    mut,
    // address=MetadataAccount::find_pda(&collection_mint.key()).0
    )]
    pub metadata_account: UncheckedAccount<'info>,

    /// CHECK:
    #[account(
    mut,
    // address=MasterEdition::find_pda(&collection_mint.key()).0
    )]
    pub master_edition: UncheckedAccount<'info>,

    #[account(
    init_if_needed,
    payer = authority,
    associated_token::mint = collection_mint,
    associated_token::authority = bridge_pda
    )]
    pub token_account: Account<'info, TokenAccount>,

    #[account(
    mut , 
    // seeds=[COLLECTION_INFO.as_bytes(), origin_chain.as_bytes() , origin_contract_address.as_bytes()],
    // bump
    )]
    pub collection_info_account: Account<'info, CollectionInfo>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(
        uri: String,
        name: String,
        symbol: String,
        origin_chain: String ,
        origin_contract_address:String)]
pub struct CreateNftInCollection<'info> {
    #[account(mut,address=pubkey!("7QHySLfCkeSBUGxtqM3WdeKv9X4YUXMeTn3gr5fRPCGU"))]
    pub signer: Signer<'info>,

    #[account(
    mut,
    seeds = [origin_chain.as_bytes() , origin_contract_address.as_bytes()],
    bump,
    )]
    pub collection_mint: Account<'info, Mint>,

    /// CHECK:
    #[account(
    mut,
    // address=MetadataAccount::find_pda(&collection_mint.key()).0
    )]
    pub collection_metadata_account: UncheckedAccount<'info>,

    /// CHECK:
    #[account(
    mut,
    // address=MasterEdition::find_pda(&collection_mint.key()).0
    )]
    pub collection_master_edition: UncheckedAccount<'info>,

    #[account(
    init_if_needed,
    payer = signer,
    mint::decimals = 0,
    mint::authority = collection_mint,
    mint::freeze_authority = collection_mint
    )]
    pub nft_mint: Account<'info, Mint>,

    /// CHECK:
    #[account(
    mut,
    // address=MetadataAccount::find_pda(&nft_mint.key()).0
    )]
    pub metadata_account: UncheckedAccount<'info>,

    /// CHECK:
    #[account(
    mut,
    // address=MasterEdition::find_pda(&nft_mint.key()).0
    )]
    pub master_edition: UncheckedAccount<'info>,

    #[account(
    init_if_needed,
    payer = signer,
    associated_token::mint = nft_mint,
    associated_token::authority = receiver
    )]
    pub token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub receiver: AccountInfo<'info>,

    #[account(
    mut , 
    seeds=[COLLECTION_INFO.as_bytes(), origin_chain.as_bytes() , origin_contract_address.as_bytes()],
    bump
    )]
    pub collection_info_account: Account<'info, CollectionInfo>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(origin_chain: String , origin_contract_address:String)]
pub struct VerifyNftInCollection<'info> {
    #[account(mut,address=pubkey!("7QHySLfCkeSBUGxtqM3WdeKv9X4YUXMeTn3gr5fRPCGU"))]
    pub signer: Signer<'info>,

    #[account(
    mut,
    seeds = [origin_chain.as_bytes() , origin_contract_address.as_bytes()],
    bump,
    )]
    pub collection_mint: Account<'info, Mint>,

    /// CHECK:
    #[account(
    mut,
    // address=MetadataAccount::find_pda(&collection_mint.key()).0
    )]
    pub collection_metadata_account: UncheckedAccount<'info>,

    /// CHECK:
    #[account(
    mut,
    // address=MasterEdition::find_pda(&collection_mint.key()).0
    )]
    pub collection_master_edition: UncheckedAccount<'info>,

    #[account(
    // mint::decimals = 0,
    // mint::authority = collection_mint,
    // mint::freeze_authority = collection_mint
    )]
    pub nft_mint: Account<'info, Mint>,

    /// CHECK:
    #[account(
    mut,
    // address=MetadataAccount::find_pda(&nft_mint.key()).0
    )]
    pub metadata_account: UncheckedAccount<'info>,

    /// CHECK:
    #[account(
    mut,
    // address=MasterEdition::find_pda(&nft_mint.key()).0
    )]
    pub master_edition: UncheckedAccount<'info>,

    #[account(
    // init_if_needed,
    // payer = signer,
    // associated_token::mint = nft_mint,
    // associated_token::authority = receiver
    )]
    pub token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub receiver: AccountInfo<'info>,

    #[account(
    mut , 
    seeds=[COLLECTION_INFO.as_bytes(), origin_chain.as_bytes() , origin_contract_address.as_bytes()],
    bump
    )]
    pub collection_info_account: Account<'info, CollectionInfo>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(origin_chain: String , origin_contract_address:String)]
pub struct StoreCollectionInfo<'info> {
    #[account(
        init_if_needed,
        seeds = [COLLECTION_INFO.as_bytes(), origin_chain.as_bytes(), origin_contract_address.as_bytes()],
        bump,
        payer = user,
        space = 8 + std::mem::size_of::<CollectionInfo>(),
    )]
    pub collection_info_account: Box<Account<'info, CollectionInfo>>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct CollectionInfo {
    pub origin_chain: String,
    pub origin_contract_address: String,
}
