// -*- coding: utf-8 -*-
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (C) 2025 Michael Büsch <m@bues.ch>

use anyhow::{self as ah, Context as _, format_err as err};
use std::{path::Path, process::Stdio};
use tempfile::tempdir;
use tokio::{fs::OpenOptions, io::AsyncWriteExt as _, process::Command};

pub async fn assemble_hex(asm_text: &str, out_hex: &Path) -> ah::Result<()> {
    let temp_dir = tempdir().context("Create temporary directory")?;

    let mut in_asm = temp_dir.path().to_path_buf();
    in_asm.push("in.asm");

    let mut in_asm_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&in_asm)
        .await
        .context("Open tmp asm file")?;

    in_asm_file
        .write_all(asm_text.as_bytes())
        .await
        .context("Write tmp asm file")?;

    let mut proc = Command::new("avra")
        .arg("-o")
        .arg(out_hex)
        .arg(in_asm)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("avra")?;
    if !proc.wait().await.context("Await avra execution")?.success() {
        return Err(err!("avra failed"));
    }

    Ok(())
}

// vim: ts=4 sw=4 expandtab
