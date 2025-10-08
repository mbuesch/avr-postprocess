// -*- coding: utf-8 -*-
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (C) 2025 Michael Büsch <m@bues.ch>

use crate::program::{Insn, InsnPatch, PartPatch, Program};
use anyhow::{self as ah, format_err as err};
use std::{collections::HashSet, sync::LazyLock};

static CALLEE_SAVED: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        "r2", "r3", "r4", "r5", "r6", "r7", "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15",
        "r16", "r17", "r28", "r29",
    ]
    .into()
});

pub async fn run(program: &mut Program) -> ah::Result<()> {
    let Some(text) = program.section_text_mut() else {
        return Err(err!("Text section not found."));
    };

    let mut rust_main_name = None;
    let mut rust_main_patched = false;
    let mut c_entry_patched = false;
    let mut c_main_patched = false;

    for part in text.parts_mut() {
        // Patch the Rust main function.
        if part.demangled().ends_with("::__avr_device_rt_main") {
            for insn in part.insns_mut() {
                if insn.name() == "push" && insn.ops().len() == 1 {
                    if CALLEE_SAVED.contains(&insn.ops()[0][..]) {
                        // This push is part of the callee-save prologue.
                        // This is not needed in the main function.
                        // Remove it.
                        insn.set_patch(Some(InsnPatch::empty()));
                    }
                } else {
                    break;
                }
            }
            rust_main_name = Some(part.name().to_string());
            rust_main_patched = true;
            break;
        }
    }

    for part in text.parts_mut() {
        if !c_entry_patched && part.demangled() == "L0^A" {
            // This is the entry from the C-rt init.
            // Directly jump to the Rust main.
            let mut new_part = part.clone_empty();
            new_part.add_insn(Insn::new(
                "rjmp",
                vec![rust_main_name.clone().unwrap()],
                None,
                0,
            ));
            part.set_patch(Some(PartPatch::new(new_part)));
            c_entry_patched = true;
        }

        if !c_main_patched && part.name() == "main" {
            // Remove the C main function.
            part.set_patch_delete_part();
            c_main_patched = true;
        }
    }

    if !rust_main_patched {
        return Err(err!("Rust main function not found."));
    }
    if !c_entry_patched {
        return Err(err!("C-rt entry function not found."));
    }
    if !c_main_patched {
        return Err(err!("C main function not found."));
    }
    Ok(())
}

// vim: ts=4 sw=4 expandtab
