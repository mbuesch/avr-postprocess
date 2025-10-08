// -*- coding: utf-8 -*-
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (C) 2025 Michael Büsch <m@bues.ch>

use crate::program::{InsnPatch, Program};
use anyhow::{self as ah, format_err as err};

pub async fn run(program: &mut Program) -> ah::Result<()> {
    let Some(text) = program.section_text_mut() else {
        return Err(err!("Text section not found."));
    };

    // Make all unused interrupt vectors point to _exit instead of __bad_interrupt.
    if let Some(part) = text.find_part_mut("__vectors") {
        for insn in part.insns_mut() {
            if insn.name() == "rjmp" && insn.ops().len() == 1 && insn.ops()[0] == "__bad_interrupt" {
                let mut pinsn = insn.clone();
                pinsn.ops_mut()[0] = "_exit".to_string();
                insn.set_patch(Some(InsnPatch::new(vec![pinsn])));
            }
        }
    } else {
        return Err(err!("Interrupt vector table not found."));
    }

    // Remove the __bad_interrupt function.
    if let Some(part) = text.find_part_mut("__bad_interrupt") {
        part.set_patch_delete_part();
    } else {
        return Err(err!("__bad_interrupt not found."));
    }

    Ok(())
}

// vim: ts=4 sw=4 expandtab
