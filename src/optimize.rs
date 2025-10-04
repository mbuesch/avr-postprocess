// -*- coding: utf-8 -*-
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (C) 2025 Michael Büsch <m@bues.ch>

use crate::program::Program;
use anyhow::{self as ah, format_err as err};
use std::collections::BTreeMap;

enum Steps {
    MainPrologue,
}

const PRIO_MAIN_PROLOGUE: i32 = 0;

async fn run_step(step: &Steps) -> ah::Result<()> {
    //TODO
    Ok(())
}

pub async fn optimize_program(_program: &mut Program, steps: &[String]) -> ah::Result<()> {
    let mut active_steps = BTreeMap::new();

    for step in steps {
        match &step[..] {
            "main-prologue" => {
                active_steps.insert(PRIO_MAIN_PROLOGUE, Steps::MainPrologue);
            }
            step => {
                return Err(err!("Unknown optimization step: {step}"));
            }
        }
    }

    while let Some((_, step)) = active_steps.pop_first() {
        run_step(&step).await?;
    }

    Ok(())
}

// vim: ts=4 sw=4 expandtab
