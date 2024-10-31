use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Kol {
    pub id: u64,  // Changed from String to u64
    #[max_len(30)]
    pub name: String,
    pub age: u8,
    #[max_len(30)]
    pub country: String,
    #[max_len(30)]
    pub pfp_type: String,
    #[max_len(100)]
    pub pfp: String,
    pub account_creation: u16,
    pub followers: u32,
    #[max_len(20)]
    pub ecosystem: String,
    pub bump: u8,
}