use crate::header;
use crate::section;

#[repr(C)]
pub struct ELF64 {
    ehdr: header::Ehdr64,
    sections: Vec<section::Section64>,
    // phdrs: Vec<program::Phdr64>,
}

impl ELF64 {
    pub fn new(elf_header: header::Ehdr64) -> Self {
        Self {
            ehdr: elf_header,
            sections: Vec::new(),
        }
    }

    pub fn condition(&mut self) {
        self.ehdr.set_shentsize(section::Shdr64::size());
        self.ehdr.set_shnum(self.sections.len() as u16);
        self.ehdr.set_shstrndx(self.sections.len() as u16 - 1);

        self.ehdr.set_ehsize(header::Ehdr64::size());
        let shoff = self.sum_section_sizes(header::Ehdr64::size() as u64);
        self.ehdr.set_shoff(shoff);

        // セクションのオフセットを揃える
        let file_offset = header::Ehdr64::size() as u64;
        self.clean_sections_offset(file_offset);

        // セクション名を揃える
        let shstrndx = self.ehdr.get_shstrndx() as usize;
        let shnum = self.ehdr.get_shnum() as usize;
        let name_count = shnum - 1;

        let mut sh_name = 1;
        for (idx, bb) in self.sections[shstrndx]
            .bytes
            .to_vec()
            .splitn(name_count, |num| *num == 0x00)
            .enumerate()
        {
            if idx == 0 || idx >= shnum {
                continue;
            }
            let b: Vec<&u8> = bb
                .iter()
                .take_while(|num| *num != &0x00)
                .collect::<Vec<&u8>>();
            self.sections[idx].header.set_name(sh_name as u32);
            sh_name += b.len() as u32 + 1;
        }
    }

    pub fn to_le_bytes(&self) -> Vec<u8> {
        let mut file_binary: Vec<u8> = Vec::new();

        let mut header_binary = self.ehdr.to_le_bytes();
        file_binary.append(&mut header_binary);

        for sct in self.sections.iter() {
            let mut section_binary = sct.bytes.clone();
            file_binary.append(&mut section_binary);
        }

        for sct in self.sections.iter() {
            let mut shdr_binary = sct.header.to_le_bytes();
            file_binary.append(&mut shdr_binary);
        }

        // TODO: Phdrs

        file_binary
    }

    pub fn section_number(&self) -> usize {
        self.sections.len()
    }

    pub fn add_section(&mut self, sct: section::Section64) {
        self.sections.push(sct);
    }

    fn clean_sections_offset(&mut self, base: u64) {
        let mut total = base;
        for section in self.sections.iter_mut() {
            let sh_offset = section.header.get_offset();
            section.header.set_offset(sh_offset + total);

            let sh_size = section.header.get_size();
            total += sh_size;
        }
    }

    fn sum_section_sizes(&self, base: u64) -> u64 {
        self.sections
            .iter()
            .fold(base, |sum, section| sum + section.bytes.len() as u64)
    }
}
