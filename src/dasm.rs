// -*- coding: utf-8 -*-
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (C) 2025 Michael Büsch <m@bues.ch>

use crate::{
    avr_deviceinfo::{AvrElfBytes, elf_avr_deviceinfo},
    program::{CodeSection, DataSection, Insn, Part, Program},
};
use anyhow::{self as ah, Context as _, format_err as err};
use regex::Regex;
use rustc_demangle::demangle;
use std::{collections::HashMap, path::Path, process::Stdio};
use tokio::process::Command;

async fn resolve_references(program: &mut Program) -> ah::Result<()> {
    let Some(device) = program.device() else {
        return Err(err!("No device info"));
    };
    let flash_size = device.flash_size;
    let flash_mask: u16 = (flash_size - 1).try_into().context("Gen flash mask")?;

    let mut addr_map = HashMap::with_capacity(flash_size.try_into()?);
    if let Some(text) = program.section_text() {
        for (p, part) in text.parts().iter().enumerate() {
            for (i, insn) in part.insns().iter().enumerate() {
                addr_map.insert(insn.addr(), (p, i));
            }
        }
    }

    let re_reloffs = Regex::new(r"^\.([\+-]\d+)$").unwrap();

    let mut rel_target = 0_u32;

    for p in 0..program.section_text().map(|t| t.parts().len()).unwrap_or(0) {
        for i in 0..program
            .section_text()
            .map(|t| t.part_at(p).insns().len())
            .unwrap_or(0)
        {
            let insn = program
                .section_text()
                .unwrap()
                .part_at(p)
                .insn_at(i)
                .clone();
            for (iop, op) in insn.ops().iter().enumerate() {
                if let Some(cap) = re_reloffs.captures(op) {
                    let offs = cap.get(1).unwrap().as_str();
                    let Ok(offs) = offs.parse::<i32>() else {
                        return Err(err!("Relative offset '{offs}' is not i32."));
                    };

                    let abs = (insn.addr() as i32 + 2 + offs) as u16;
                    let abs = abs & flash_mask;

                    let Some(target) = addr_map.get(&abs) else {
                        return Err(err!("Relative offset '{offs}' target not found."));
                    };

                    let target_label = match program
                        .section_text()
                        .unwrap()
                        .part_at(target.0)
                        .insn_at(target.1)
                        .label()
                    {
                        Some(label) => label.to_string(),
                        None => {
                            let label = format!("__reltgt{rel_target}");
                            rel_target += 1;
                            program
                                .section_text_mut()
                                .unwrap()
                                .part_at_mut(target.0)
                                .insn_at_mut(target.1)
                                .set_label(Some(label.to_string()));
                            label
                        }
                    };

                    program
                        .section_text_mut()
                        .unwrap()
                        .part_at_mut(p)
                        .insn_at_mut(i)
                        .set_op(iop, target_label);
                }
            }
        }
    }

    Ok(())
}

fn sanitize_label(label: &str) -> String {
    let mut label = label.to_string();
    label = label.replace('.', "__dot__");
    label = label.replace('$', "__dollar__");
    label = label.replace('^', "__caret__");
    label
}

async fn process_dasm(program: &mut Program, raw: &str) -> ah::Result<()> {
    let re_format = Regex::new(r"^.*file format elf32-avr$").unwrap();
    let re_section = Regex::new(r"^Disassembly of section ([^:]+):$").unwrap();
    let re_symbol = Regex::new(r"^([0-9a-fA-F]{8})\s+<([^>]+)>:$").unwrap();
    let re_insn =
        Regex::new(r"^([0-9a-fA-F]+):\s+((?:[0-9a-fA-F]{2} )+)\s+(\S+)\s*([^;]*)").unwrap();

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(cap) = re_insn.captures(line) {
            let addr = cap.get(1).unwrap().as_str().trim();
            let _raw_code = cap.get(2).unwrap().as_str().trim();
            let name = cap.get(3).unwrap().as_str().trim();
            let opers = cap.get(4).unwrap().as_str().trim();

            let Ok(addr_int) = u16::from_str_radix(addr, 16) else {
                return Err(err!(
                    "Failed to parse address '{addr}' of '{name} {opers}'."
                ));
            };
            let opers_list = opers.split(',').map(|o| o.trim().to_string()).collect();

            let insn = Insn::new(name, opers_list, None, addr_int);
            if let Some(sect) = program.section_text_mut() {
                if let Some(part) = sect.cur_part_mut() {
                    part.add_insn(insn);
                } else {
                    return Err(err!(
                        "Got '{name} {opers}' instruction at addr {addr}, \
                        but we are not in a function."
                    ));
                }
            } else {
                return Err(err!(
                    "Got '{name} {opers}' instruction at addr {addr}, \
                    but we are not not in a section."
                ));
            }
        } else if let Some(cap) = re_symbol.captures(line) {
            let addr = cap.get(1).unwrap().as_str().trim();
            let name = cap.get(2).unwrap().as_str().trim();
            if name.starts_with(".L") {
                // ignore
            } else if let Some(sect) = program.section_text_mut() {
                let san_name = sanitize_label(name);
                let demangled = format!("{:#}", demangle(name));
                sect.add_part(Part::new(&san_name, &demangled));
            } else {
                return Err(err!(
                    "Got '{name}' part at addr {addr}, \
                    but we are not not in a section."
                ));
            }
        } else if let Some(cap) = re_section.captures(line) {
            let name = cap.get(1).unwrap().as_str().trim();
            if name != ".text" {
                return Err(err!(
                    "The disassembled section is not '.text' but '{name}'."
                ));
            }
            program.set_section_text(Some(CodeSection::new(name)));
        } else if let Some(_cap) = re_format.captures(line) {
            // ignore
        } else {
            eprintln!("WARNING: Unknown disassembly line: {line}");
        }
    }

    resolve_references(program).await?;

    Ok(())
}

pub async fn disassemble_elf_text(program: &mut Program, file: &Path) -> ah::Result<()> {
    let proc = Command::new("avr-objdump")
        .arg("--disassemble")
        .arg("-j")
        .arg(".text")
        .arg(file)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("avr-objdump --disassemble")?;
    let out = proc
        .wait_with_output()
        .await
        .context("Read output of: avr-objdump --disassemble")?;
    if !out.status.success() {
        return Err(err!("avr-objdump --disassemble {file:?} failed"));
    }
    let asm_raw = String::from_utf8(out.stdout).context("Asm UTF-8 conversion")?;
    process_dasm(program, &asm_raw).await?;
    Ok(())
}

pub async fn extract_elf_data_section(
    program: &mut Program,
    elf: &AvrElfBytes<'_>,
) -> ah::Result<()> {
    let shdr = elf
        .section_header_by_name(".data")
        .context("Parse section table")?
        .context("Get .data section")?;
    let sdata = elf
        .section_data(&shdr)
        .context("Get .data section content")?;
    let sdata = sdata.0;
    program.set_section_data(Some(DataSection::new(".data".to_string(), sdata.to_vec())));
    Ok(())
}

pub async fn extract_elf_deviceinfo(
    program: &mut Program,
    elf: &AvrElfBytes<'_>,
) -> ah::Result<()> {
    program.set_device(Some(elf_avr_deviceinfo(elf)?));
    Ok(())
}

pub async fn extract_elf_data(program: &mut Program, file: &Path) -> ah::Result<()> {
    let data = std::fs::read(file).context("Read ELF input file")?;
    let elf = AvrElfBytes::minimal_parse(&data).context("Parse ELF input file")?;
    extract_elf_data_section(program, &elf).await?;
    extract_elf_deviceinfo(program, &elf).await?;
    Ok(())
}

// vim: ts=4 sw=4 expandtab
