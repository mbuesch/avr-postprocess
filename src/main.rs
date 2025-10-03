// -*- coding: utf-8 -*-
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (C) 2025 Michael Büsch <m@bues.ch>

use crate::{
    asm::assemble_hex,
    dasm::{disassemble_elf_text, extract_data},
    optimize::optimize_program,
};
use anyhow::{self as ah, Context as _};
use clap::Parser;
use std::path::PathBuf;

mod asm;
mod dasm;
mod optimize;
mod program;

pub struct AvrHw {
    name: String,
    flash_mask: u16,
}

impl AvrHw {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn flash_mask(&self) -> u16 {
        self.flash_mask
    }

    pub fn flash_size(&self) -> usize {
        self.flash_mask() as usize + 1
    }
}

#[derive(Parser, Debug)]
struct Opts {
    input_elf: PathBuf,
    output_hex: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ah::Result<()> {
    let opts = Opts::parse();

    //TODO
    let hw = AvrHw {
        name: "attiny861a".to_string(),
        flash_mask: (1024 * 8) - 1,
    };

    let mut program = disassemble_elf_text(&opts.input_elf, &hw)
        .await
        .context("Disassemble program")?;

    extract_data(&mut program, &opts.input_elf, &hw)
        .await
        .context("Extract .data section")?;

    optimize_program(&mut program, &hw)
        .await
        .context("Optimize program")?;

    assemble_hex(&program, &opts.output_hex, &hw)
        .await
        .context("Assemble program")?;

    Ok(())
}

// vim: ts=4 sw=4 expandtab
