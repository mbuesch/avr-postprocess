// -*- coding: utf-8 -*-
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (C) 2025 Michael Büsch <m@bues.ch>

use crate::{AvrHw, program::Program};
use anyhow::{self as ah};

pub async fn optimize_program(_program: &mut Program, _hw: &AvrHw) -> ah::Result<()> {
    //TODO
    Ok(())
}

// vim: ts=4 sw=4 expandtab
