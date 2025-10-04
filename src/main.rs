// -*- coding: utf-8 -*-
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (C) 2025 Michael Büsch <m@bues.ch>

use crate::{
    asm::assemble_hex,
    dasm::{disassemble_elf_text, extract_elf_data},
    optimize::optimize_program,
    program::Program,
};
use anyhow::{self as ah, Context as _};
use clap::Parser;
use std::path::PathBuf;

mod asm;
mod avr_deviceinfo;
mod dasm;
mod optimize;
mod program;

#[derive(Parser, Debug)]
struct Opts {
    input_elf: PathBuf,

    output: PathBuf,

    #[arg(long)]
    no_optimize: bool,

    #[arg(long)]
    output_asm: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ah::Result<()> {
    let opts = Opts::parse();

    let mut program = Program::new();

    extract_elf_data(&mut program, &opts.input_elf)
        .await
        .context("Extract .data section")?;

    disassemble_elf_text(&mut program, &opts.input_elf)
        .await
        .context("Disassemble program")?;

    if !opts.no_optimize {
        optimize_program(&mut program)
            .await
            .context("Optimize program")?;
    }

    if opts.output_asm {
        //TODO
        println!("{}", program.to_asm()?)
    } else {
        assemble_hex(&program, &opts.output)
            .await
            .context("Assemble program")?;
    }

    Ok(())
}

// vim: ts=4 sw=4 expandtab
