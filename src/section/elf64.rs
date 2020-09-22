//! Type definitions for 64-bit ELF binaries.

use crate::section::section_type;
use crate::*;

use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
pub struct Section64 {
    pub name: String,
    pub header: Shdr64,

    /// for normal section
    pub bytes: Option<Vec<u8>>,

    /// for symbol table
    pub symbols: Option<Vec<symbol::Symbol64>>,

    /// for rela symbol table
    pub rela_symbols: Option<Vec<relocation::Rela64>>,
}

impl Section64 {
    pub fn new(section_name: String, shdr: Shdr64) -> Self {
        Self {
            name: section_name,
            header: shdr,
            bytes: None,
            symbols: None,
            rela_symbols: None,
        }
    }

    pub fn write_byte_to_index(&mut self, byte: u8, idx: usize) {
        if let Some(ref mut bytes) = self.bytes {
            bytes[idx] = byte;
        }
    }

    /// create binary without header
    pub fn to_le_bytes(&self) -> Vec<u8> {
        match self.header.get_type() {
            section_type::TYPE::SYMTAB => {
                let mut bytes = Vec::new();

                for sym in self.symbols.as_ref().unwrap().iter() {
                    bytes.append(&mut sym.to_le_bytes());
                }
                bytes
            }
            section_type::TYPE::RELA => {
                let mut bytes = Vec::new();

                for rela in self.rela_symbols.as_ref().unwrap().iter() {
                    bytes.append(&mut rela.to_le_bytes());
                }

                if let Some(bts) = &self.bytes {
                    bytes.append(&mut bts.clone());
                }

                bytes
            }
            _ => self.bytes.as_ref().unwrap().clone(),
        }
    }

    pub fn new_null_section() -> Self {
        let mut null_section = Self::new(String::new(), Default::default());
        null_section.bytes = Some(Vec::new());
        null_section
    }
}

#[derive(Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)]
pub struct Shdr64 {
    pub sh_name: Elf64Word,
    pub sh_type: Elf64Word,
    pub sh_flags: Elf64Xword,
    pub sh_addr: Elf64Addr,
    pub sh_offset: Elf64Off,
    pub sh_size: Elf64Xword,
    pub sh_link: Elf64Word,
    pub sh_info: Elf64Word,
    pub sh_addralign: Elf64Xword,
    pub sh_entsize: Elf64Xword,
}

impl Default for Shdr64 {
    fn default() -> Self {
        Self {
            sh_name: 0,
            sh_type: 0,
            sh_flags: 0,
            sh_addr: 0,
            sh_offset: 0,
            sh_size: 0,
            sh_link: 0,
            sh_info: 0,
            sh_addralign: 0,
            sh_entsize: 0,
        }
    }
}

#[allow(dead_code)]
impl Shdr64 {
    pub fn size() -> Elf64Half {
        0x40
    }

    // getter
    pub fn get_type(&self) -> section_type::TYPE {
        section_type::TYPE::from(self.sh_type)
    }
    // setter
    pub fn set_type(&mut self, ty: section_type::TYPE) {
        self.sh_type = ty.to_bytes();
    }

    /// Create Vec<u8> from this.
    ///
    /// # Examples
    ///
    /// ```
    /// use elf_utilities::section::Shdr64;
    /// let null_sct : Shdr64 = Default::default();
    ///
    /// assert_eq!([0].repeat(Shdr64::size() as usize), null_sct.to_le_bytes());
    /// ```
    pub fn to_le_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn deserialize(buf: &[u8], start: usize) -> Result<Self, Box<dyn std::error::Error>>{
        // bincode::ErrorKindをトレイトオブジェクトとするため,この冗長な書き方が必要
        match bincode::deserialize(&buf[start..]){
            Ok(header) => Ok(header),
            Err(e) => Err(e),
        }
    }
}
