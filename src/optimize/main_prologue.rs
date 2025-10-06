// -*- coding: utf-8 -*-
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (C) 2025 Michael Büsch <m@bues.ch>

use crate::program::{Patch, Program};
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
    for part in text.parts_mut() {
        if part.demangled().ends_with("::__avr_device_rt_main") {
            for insn in part.insns_mut() {
                if insn.name() == "push" && insn.ops().len() == 1 {
                    if CALLEE_SAVED.contains(&insn.ops()[0][..]) {
                        // This push is part of the callee-save prologue.
                        // This is not needed in the main function.
                        // Remove it.
                        insn.set_patch(Some(Patch::empty()));
                    }
                } else {
                    break;
                }
            }
            return Ok(());
        }
    }
    Err(err!("Main function not found."))
}

// vim: ts=4 sw=4 expandtab
