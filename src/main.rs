// -*- coding: utf-8 -*-
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (C) 2025 Michael Büsch <m@bues.ch>

use crate::{
    asm::assemble_hex,
    dasm::{disassemble_elf_text, extract_elf_data},
    patch::patch_program,
    program::Program,
};
use anyhow::{self as ah, Context as _};
use clap::Parser;
use std::path::PathBuf;
use tokio::{fs::OpenOptions, io::AsyncWriteExt as _};

mod abi;
mod asm;
mod avr_deviceinfo;
mod dasm;
mod patch;
mod program;

#[derive(Parser, Debug)]
struct Opts {
    input_elf: PathBuf,

    output: PathBuf,

    #[arg(short = 'P', long)]
    patch: Vec<String>,

    #[arg(short = 'A', long)]
    dump_asm: Option<String>,
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

    program.fixup_data_load_addr().context("Fixup .data")?;

    patch_program(&mut program, &opts.patch)
        .await
        .context("Patch program")?;

    let asm_text = program
        .to_asm()
        .context("Convert program to assembly code")?;

    assemble_hex(&asm_text, &opts.output)
        .await
        .context("Assemble program")?;

    if let Some(dump_asm) = &opts.dump_asm {
        if dump_asm == "-" {
            println!("\n\n; Begin: Assembly listing");
            println!("{asm_text}");
            println!("; End: Assembly listing");
        } else {
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(dump_asm)
                .await
                .context("Open --dump-asm file")?
                .write_all(asm_text.as_bytes())
                .await
                .context("Write --dump-asm file")?
        }
    }

    Ok(())
}

// vim: ts=4 sw=4 expandtab
