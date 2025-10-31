// -*- coding: utf-8 -*-
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (C) 2025 Michael Büsch <m@bues.ch>

use std::{collections::HashSet, sync::LazyLock};

static CALLEE_SAVED: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    [
        "r2", "r3", "r4", "r5", "r6", "r7", "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15",
        "r16", "r17", "r28", "r29",
    ]
    .into()
});

pub fn reg_is_callee_saved(reg: &str) -> bool {
    CALLEE_SAVED.contains(reg)
}

// vim: ts=4 sw=4 expandtab
