use crate::header::osabi;
use crate::*;

pub const ELF_MAGIC_NUMBER: u128 = (0x7f454c46) << (12 * 8);
#[repr(C)]
pub struct Ehdr64 {
    e_ident: u128,
    e_type: Elf64Half,
    e_machine: Elf64Half,
    e_version: Elf64Word,
    e_entry: Elf64Addr,
    e_phoff: Elf64Off,
    e_shoff: Elf64Off,
    e_flags: Elf64Word,
    e_ehsize: Elf64Half,
    e_phentsize: Elf64Half,
    e_phnum: Elf64Half,
    e_shentsize: Elf64Half,
    e_shnum: Elf64Half,
    e_shstrndx: Elf64Half,
}

impl Ehdr64 {
    pub fn new() -> Self {
        Self {
            e_ident: ELF_MAGIC_NUMBER,
            e_type: 0,
            e_machine: 0,
            e_version: 0,
            e_entry: 0,
            e_phoff: 0,
            e_shoff: 0,
            e_flags: 0,
            e_ehsize: 0,
            e_phentsize: 0,
            e_phnum: 0,
            e_shentsize: 0,
            e_shnum: 0,
            e_shstrndx: 0,
        }
    }
    // getter
    pub fn get_identification(&self) -> u128 {
        self.e_ident
    }

    // setter
    pub fn set_class(&mut self, class: ELF64CLASS) {
        self.e_ident |= class.to_identifier();
    }
    pub fn set_data(&mut self, data: ELF64DATA) {
        self.e_ident |= data.to_identifier();
    }
    pub fn set_version(&mut self, version: ELF64VERSION) {
        self.e_ident |= version.to_identifier();
    }
    pub fn set_osabi(&mut self, osabi: osabi::ELF64OSABI) {
        self.e_ident |= osabi.to_identifier();
    }
}

#[repr(C)]
pub struct Ehdr64Builder {
    e_ident: u128,
    e_type: Elf64Half,
    e_machine: Elf64Half,
    e_version: Elf64Word,
    e_entry: Elf64Addr,
    e_phoff: Elf64Off,
    e_shoff: Elf64Off,
    e_flags: Elf64Word,
    e_ehsize: Elf64Half,
    e_phentsize: Elf64Half,
    e_phnum: Elf64Half,
    e_shentsize: Elf64Half,
    e_shnum: Elf64Half,
    e_shstrndx: Elf64Half,
}
impl Ehdr64Builder {
    pub fn new() -> Self {
        Self {
            e_ident: ELF_MAGIC_NUMBER,
            e_type: 0,
            e_machine: 0,
            e_version: 0,
            e_entry: 0,
            e_phoff: 0,
            e_shoff: 0,
            e_flags: 0,
            e_ehsize: 0,
            e_phentsize: 0,
            e_phnum: 0,
            e_shentsize: 0,
            e_shnum: 0,
            e_shstrndx: 0,
        }
    }
    pub fn class(&mut self, class: ELF64CLASS) -> &mut Self {
        self.set_class(class);
        self
    }
    pub fn data(&mut self, data: ELF64DATA) -> &mut Self {
        self.set_data(data);
        self
    }
    pub fn version(&mut self, version: ELF64VERSION) -> &mut Self {
        self.set_version(version);
        self
    }
    pub fn osabi(&mut self, osabi: osabi::ELF64OSABI) -> &mut Self {
        self.set_osabi(osabi);
        self
    }
    pub fn finalize(&self) -> Ehdr64 {
        Ehdr64 {
            e_ident: self.e_ident,
            e_type: self.e_type,
            e_machine: self.e_machine,
            e_version: self.e_version,
            e_entry: self.e_entry,
            e_phoff: self.e_phoff,
            e_shoff: self.e_shoff,
            e_flags: self.e_flags,
            e_ehsize: self.e_ehsize,
            e_phentsize: self.e_phentsize,
            e_phnum: self.e_phnum,
            e_shentsize: self.e_shentsize,
            e_shnum: self.e_shnum,
            e_shstrndx: self.e_shstrndx,
        }
    }

    fn set_class(&mut self, class: ELF64CLASS) {
        self.e_ident |= class.to_identifier();
    }
    fn set_data(&mut self, data: ELF64DATA) {
        self.e_ident |= data.to_identifier();
    }
    fn set_version(&mut self, version: ELF64VERSION) {
        self.e_ident |= version.to_identifier();
    }
    fn set_osabi(&mut self, osabi: osabi::ELF64OSABI) {
        self.e_ident |= osabi.to_identifier();
    }
}

pub enum ELF64CLASS {
    // invalid class
    CLASSNone,
    // 32bit objects
    CLASS32,
    // 64bit objects
    CLASS64,
    CLASSNUM,

    // for architecture-specific-value
    ANY(u8),
}

impl ELF64CLASS {
    pub fn to_identifier(&self) -> u128 {
        let byte = match self {
            Self::CLASSNone => 0,
            Self::CLASS32 => 1,
            Self::CLASS64 => 2,
            Self::CLASSNUM => 3,
            Self::ANY(b) => *b,
        };
        Self::shift_position(byte)
    }
    fn shift_position(byte: u8) -> u128 {
        (byte as u128) << 88
    }
}

pub enum ELF64DATA {
    // invalid data encoding
    DATANONE,
    // 2's complement little endian
    DATA2LSB,
    // 2's complement big endian
    DATA2MSB,
    DATA2NUM,

    // for architecture-specific-value
    ANY(u8),
}

impl ELF64DATA {
    pub fn to_identifier(&self) -> u128 {
        let byte = match self {
            Self::DATANONE => 0,
            Self::DATA2LSB => 1,
            Self::DATA2MSB => 2,
            Self::DATA2NUM => 3,
            Self::ANY(c) => *c,
        };
        Self::shift_position(byte)
    }
    fn shift_position(byte: u8) -> u128 {
        (byte as u128) << 80
    }
}

pub enum ELF64VERSION {
    // value must be 1
    VERSIONCURRENT,

    // for architecture-specific-value
    ANY(u8),
}

impl ELF64VERSION {
    pub fn to_identifier(&self) -> u128 {
        let byte = match self {
            Self::VERSIONCURRENT => 1,
            Self::ANY(c) => *c,
        };
        Self::shift_position(byte)
    }
    fn shift_position(byte: u8) -> u128 {
        (byte as u128) << 72
    }
}
