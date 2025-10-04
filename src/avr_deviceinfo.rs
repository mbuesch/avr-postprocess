// -*- coding: utf-8 -*-
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright (C) 2025 Michael Büsch <m@bues.ch>

use anyhow::{self as ah, Context as _, format_err as err};
use elf::string_table::StringTable;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AvrDeviceInfoDesc {
    pub flash_start: u32,
    pub flash_size: u32,
    pub sram_start: u32,
    pub sram_size: u32,
    pub eeprom_start: u32,
    pub eeprom_size: u32,
    pub device_name: String,
}

const U32LEN: usize = u32::BITS as usize / 8;

impl AvrDeviceInfoDesc {
    pub fn from_bytes(data: &[u8]) -> ah::Result<Self> {
        if data.len() < 8 * U32LEN {
            return Err(err!(".note.gnu.avr.deviceinfo descriptor is too small."));
        }
        let flash_start = u32::from_le_bytes(data[0..4].try_into()?);
        let flash_size = u32::from_le_bytes(data[4..8].try_into()?);
        let sram_start = u32::from_le_bytes(data[8..12].try_into()?);
        let sram_size = u32::from_le_bytes(data[12..16].try_into()?);
        let eeprom_start = u32::from_le_bytes(data[16..20].try_into()?);
        let eeprom_size = u32::from_le_bytes(data[20..24].try_into()?);
        let _offset_table_size = u32::from_le_bytes(data[24..28].try_into()?);
        // offset_table_size seems to be incorrect. Why is it 8?
        // Just assume there is exactly one entry.
        let offset_table_0: usize = u32::from_le_bytes(data[28..32].try_into()?).try_into()?;

        let stab = StringTable::new(&data[32..]);
        let s = stab
            .get(offset_table_0)
            .context("Parse .note.gnu.avr.deviceinfo string table")?;
        let device_name = s.to_string();

        Ok(Self {
            flash_start,
            flash_size,
            sram_start,
            sram_size,
            eeprom_start,
            eeprom_size,
            device_name,
        })
    }
}

impl TryFrom<&[u8]> for AvrDeviceInfoDesc {
    type Error = ah::Error;

    fn try_from(data: &[u8]) -> ah::Result<Self> {
        Self::from_bytes(data)
    }
}

// vim: ts=4 sw=4 expandtab
