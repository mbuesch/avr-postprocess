// -*- coding: utf-8 -*-
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (C) 2025 Michael Büsch <m@bues.ch>

use crate::avr_deviceinfo::AvrDeviceInfoDesc;
use anyhow::{self as ah, format_err as err};

#[derive(Clone, Debug)]
pub struct Insn {
    name: String,
    ops: Vec<String>,
    label: Option<String>,
    addr: u16,
}

impl Insn {
    pub fn new(name: &str, ops: Vec<String>, label: Option<String>, addr: u16) -> Self {
        Self {
            name: name.to_string(),
            label,
            ops,
            addr,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ops(&self) -> &[String] {
        &self.ops
    }

    pub fn ops_mut(&mut self) -> &mut [String] {
        &mut self.ops
    }

    pub fn set_op(&mut self, index: usize, op: String) {
        self.ops[index] = op;
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn set_label(&mut self, label: Option<String>) {
        self.label = label;
    }

    pub fn addr(&self) -> u16 {
        self.addr
    }
}

impl std::fmt::Display for Insn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        if let Some(label) = &self.label {
            write!(f, "{}: ", label)?;
        }
        write!(f, "{}", self.name())?;
        for (i, op) in self.ops().iter().enumerate() {
            if i == 0 {
                write!(f, " {op}")?
            } else {
                write!(f, ", {op}")?
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Part {
    name: String,
    demangled: String,
    insns: Vec<Insn>,
}

impl Part {
    pub fn new(name: &str, demangled: &str) -> Self {
        Self {
            name: name.to_string(),
            demangled: demangled.to_string(),
            insns: vec![],
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn demangled(&self) -> &str {
        &self.demangled
    }

    pub fn add_insn(&mut self, insn: Insn) {
        self.insns.push(insn);
    }

    pub fn insns(&self) -> &[Insn] {
        &self.insns
    }

    pub fn insns_mut(&mut self) -> &mut [Insn] {
        &mut self.insns
    }

    pub fn insn_at(&self, index: usize) -> &Insn {
        &self.insns[index]
    }

    pub fn insn_at_mut(&mut self, index: usize) -> &mut Insn {
        &mut self.insns[index]
    }
}

#[derive(Clone, Debug)]
pub struct CodeSection {
    name: String,
    parts: Vec<Part>,
}

impl CodeSection {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            parts: vec![],
        }
    }

    #[allow(unused)]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn cur_part_mut(&mut self) -> Option<&mut Part> {
        self.parts.last_mut()
    }

    pub fn parts(&self) -> &[Part] {
        &self.parts
    }

    pub fn part_at(&self, index: usize) -> &Part {
        &self.parts[index]
    }

    pub fn part_at_mut(&mut self, index: usize) -> &mut Part {
        &mut self.parts[index]
    }

    pub fn add_part(&mut self, part: Part) {
        self.parts.push(part);
    }

    #[allow(unused)]
    pub fn find_part(&self, name: &str) -> Option<&Part> {
        self.parts.iter().find(|p| p.name() == name)
    }

    pub fn find_part_mut(&mut self, name: &str) -> Option<&mut Part> {
        self.parts.iter_mut().find(|p| p.name() == name)
    }
}

#[derive(Clone, Debug)]
pub struct DataSection {
    name: String,
    data: Vec<u8>,
}

impl DataSection {
    pub fn new(name: String, data: Vec<u8>) -> Self {
        Self { name, data }
    }

    #[allow(unused)]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    text: Option<CodeSection>,
    data: Option<DataSection>,
    device: Option<AvrDeviceInfoDesc>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            text: None,
            data: None,
            device: None,
        }
    }

    pub fn set_section_text(&mut self, text: Option<CodeSection>) {
        self.text = text;
    }

    pub fn section_text(&self) -> Option<&CodeSection> {
        self.text.as_ref()
    }

    pub fn section_text_mut(&mut self) -> Option<&mut CodeSection> {
        self.text.as_mut()
    }

    pub fn set_section_data(&mut self, data: Option<DataSection>) {
        self.data = data;
    }

    pub fn section_data(&self) -> Option<&DataSection> {
        self.data.as_ref()
    }

    #[allow(unused)]
    pub fn section_data_mut(&mut self) -> Option<&mut DataSection> {
        self.data.as_mut()
    }

    pub fn set_device(&mut self, device: Option<AvrDeviceInfoDesc>) {
        self.device = device;
    }

    pub fn device(&self) -> Option<&AvrDeviceInfoDesc> {
        self.device.as_ref()
    }

    pub fn to_asm(&self) -> ah::Result<String> {
        if let Some(device) = self.device.as_ref() {
            Ok(format!(".device {}\n\n{}", device.device_name, self))
        } else {
            Err(err!("No device info."))
        }
    }

    pub fn fixup_data_load_addr(&mut self) -> ah::Result<()> {
        let Some(text) = self.section_text_mut() else {
            return Err(err!("No .text section."));
        };
        let Some(fcopy) = text.find_part_mut("__do_copy_data") else {
            return Err(err!(
                "Function '__do_copy_data' not found in .text section."
            ));
        };

        let mut zl = false;
        let mut zh = false;
        for insn in fcopy.insns_mut() {
            if insn.name() == "ldi" && insn.ops().len() == 2 && insn.ops()[0] == "r30" {
                insn.ops_mut()[1] = "low(____section_data__ * 2)".to_string();
                zl = true;
            } else if insn.name() == "ldi" && insn.ops().len() == 2 && insn.ops()[0] == "r31" {
                insn.ops_mut()[1] = "high(____section_data__ * 2)".to_string();
                zh = true;
            } else if zl || zh {
                break;
            }
            if zl && zh {
                return Ok(());
            }
        }

        Err(err!("__do_copy_data: flash load address not found."))
    }
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        if let Some(sect) = self.section_text() {
            writeln!(f, ".cseg ;flash")?;
            writeln!(f, "____section_text__:")?;
            for part in sect.parts() {
                if part.name() == part.demangled() {
                    writeln!(f, "{}:", part.name())?;
                } else {
                    writeln!(f, "{}: ; {}", part.name(), part.demangled())?;
                }
                for insn in part.insns() {
                    writeln!(f, "    {insn}")?;
                }
            }
        }
        if let Some(sect) = self.section_data() {
            writeln!(f)?;
            writeln!(f, ".cseg ;flash")?;
            writeln!(f, "____section_data__:")?;
            for i in 0..sect.data().len() {
                if i % 8 == 0 {
                    if i != 0 {
                        writeln!(f)?;
                    }
                    write!(f, ".db 0x{:02X}", sect.data()[i])?;
                } else {
                    write!(f, ", 0x{:02X}", sect.data()[i])?;
                }
            }
        }
        Ok(())
    }
}

// vim: ts=4 sw=4 expandtab
