// -*- coding: utf-8 -*-
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (C) 2025 Michael Büsch <m@bues.ch>

use crate::program::Program;
use anyhow::{self as ah, Context as _, format_err as err};
use std::collections::BTreeMap;

macro_rules! define_optimizers {
    (
        $(
            {
                module: $module:ident,
                name: $name:literal,
                prio: $prio:literal,
            }
        ),*
    ) => {
        paste::paste! {
            $(
                mod $module;
            )*

            #[allow(non_camel_case_types)]
            enum Steps {
                $(
                    $module,
                )*
            }

            $(
                #[allow(non_upper_case_globals)]
                const [<PRIO_ $module>]: i32 = $prio;
            )*

            async fn run_step(program: &mut Program, step: &Steps) -> ah::Result<()> {
                match step {
                    $(
                        Steps::$module => $module::run(program)
                            .await
                            .context($name),
                    )*
                }
            }

            pub async fn optimize_program(program: &mut Program, steps: &[String]) -> ah::Result<()> {
                let mut active_steps = BTreeMap::new();

                for step in steps {
                    match &step[..] {
                        $(
                            $name => {
                                active_steps.insert([<PRIO_ $module>], Steps::$module);
                            }
                        )*
                        step => {
                            return Err(err!("Unknown optimization step: {step}"));
                        }
                    }
                }

                while let Some((_, step)) = active_steps.pop_first() {
                    run_step(program, &step).await?;
                }

                Ok(())
            }
        }
    }
}

define_optimizers! {
    {
        module: main_prologue,
        name: "main-prologue",
        prio: 0,
    }, {
        module: bad_interrupt_exit,
        name: "bad-interrupt-exit",
        prio: 1,
    }
}

// vim: ts=4 sw=4 expandtab
