use crate::*;
use header::ELFHeader;
use section::{Contents, Section};
use std::fs::File;
use std::io::Read;

use thiserror::Error as TError;

#[derive(TError, Debug)]
pub enum ReadELFError {
    #[error("input file `{file_path}` is not an ELF file")]
    NotELF { file_path: String },
    #[error("can't parse elf header => `{k}`")]
    CantParseELFHeader { k: Box<dyn std::error::Error> },
    #[error("can't parse section header => `{k}`")]
    CantParseSectionHeader { k: Box<dyn std::error::Error> },
    #[error("can't parse program header => `{k}`")]
    CantParseProgramHeader { k: Box<dyn std::error::Error> },
    #[error("can't parse symbol => `{k}`")]
    CantParseSymbol { k: Box<dyn std::error::Error> },
}

pub fn read_elf64(file_path: &str) -> Result<file::ELF64, Box<dyn std::error::Error>> {
    read_elf(file_path)
}
pub fn read_elf32(file_path: &str) -> Result<file::ELF32, Box<dyn std::error::Error>> {
    read_elf(file_path)
}

/// read ELF and construct `file::ELF`
fn read_elf<F: file::ELF>(file_path: &str) -> Result<F, Box<dyn std::error::Error>> {
    let mut f = File::open(file_path)?;
    let mut buf = Vec::new();
    let _ = f.read_to_end(&mut buf);

    let _ = check_elf_magic(file_path, &buf[..4])?;

    let elf_header: F::Header = parse_elf_header(&buf);
    let phdr_table_exists = elf_header.program_header_table_exists();

    let mut elf_file = F::new(elf_header);

    let sections = read_elf_sections(
        elf_file.header().section_number(),
        elf_file.header().section_offset(),
        &buf,
    )?;
    elf_file.update_sections(sections);

    if phdr_table_exists {
        let segments = read_elf_segments(
            elf_file.header().segment_number(),
            elf_file.header().segment_offset(),
            &buf,
        )?;
        elf_file.update_segments(segments);
    }

    set_sections_name_shstrtab(
        elf_file.header().section_name_table_idx(),
        elf_file.header().section_number(),
        elf_file.sections_as_mut(),
    );

    for idx in 0..elf_file.header().section_number() {
        if elf_file.sections_as_mut()[idx].section_type() != section::Type::SymTab
            && elf_file.sections_as_mut()[idx].section_type() != section::Type::DynSym
        {
            continue;
        }

        let related_string_table_index = elf_file.sections_as_mut()[idx].section_link();
        let name_bytes = elf_file.sections_as_mut()[related_string_table_index].clone_raw_binary();

        let symbol_number = elf_file.sections_as_mut()[idx].symbol_number();
        for sym_idx in 0..symbol_number {
            elf_file.sections_as_mut()[idx].update_symbol_name(sym_idx, &name_bytes);
        }
    }

    Ok(elf_file)
}

fn read_elf_sections<S: section::Section>(
    section_number: usize,
    sht_offset: usize,
    buf: &[u8],
) -> Result<Vec<S>, Box<dyn std::error::Error>> {
    let mut sections: Vec<S> = Vec::new();

    for sct_idx in 0..section_number {
        let header_start = sht_offset + S::header_size() * sct_idx;
        let shdr = S::header_deserialize(buf, header_start)?;

        let mut sct = S::new(shdr);
        let section_type = sct.section_type();

        if section_type != section::Type::NoBits {
            let section_offset = sct.offset();
            let section_raw_contents =
                buf[section_offset..section_offset + sct.section_size() as usize].to_vec();
            // とりあえずRawで保持しておいて，後で変換する
            sct.update_contents_from_raw_bytes(section_raw_contents);
        }

        sections.push(sct);
    }

    Ok(sections)
}

fn read_elf_segments<T: segment::Segment>(
    segment_number: usize,
    pht_offset: usize,
    buf: &[u8],
) -> Result<Vec<T>, Box<dyn std::error::Error>> {
    let mut segments: Vec<T> = Vec::new();

    for seg_idx in 0..segment_number {
        let header_start = pht_offset as usize + T::header_size() * seg_idx;
        let phdr = T::header_deserialize(buf, header_start)?;

        let seg = T::new(phdr);
        segments.push(seg);
    }

    Ok(segments)
}

fn set_sections_name_shstrtab<T: section::Section>(
    shstrndx: usize,
    section_number: usize,
    sections: &mut Vec<T>,
) {
    for idx in 0..section_number {
        if idx == 0 || idx >= section_number {
            continue;
        }

        let name_idx = sections[idx].name_idx();

        let name_bytes = sections[shstrndx].clone_contents();
        let name_bytes: Vec<u8> = name_bytes.clone_raw_binary()[name_idx as usize..]
            .to_vec()
            .iter()
            .take_while(|byte| **byte != 0x00)
            .copied()
            .collect();

        sections[idx].update_name(std::str::from_utf8(&name_bytes).unwrap().to_string());
    }
}

fn check_elf_magic(file_path: &str, buf: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(buf.len(), 4);

    if buf[0] != 0x7f || buf[1] != 0x45 || buf[2] != 0x4c || buf[3] != 0x46 {
        return Err(Box::new(ReadELFError::NotELF {
            file_path: file_path.to_string(),
        }));
    }

    Ok(())
}

fn parse_elf_header<T: header::ELFHeader>(buf: &[u8]) -> T {
    T::deserialize(buf)
}

#[cfg(test)]
mod parse_tests {
    use super::*;

    #[test]
    fn check_elf_magic_test() {
        assert!(check_elf_magic("", &[0x7f, 0x45, 0x4c, 0x46]).is_ok());
        assert!(check_elf_magic("", &[0x7f, 0x45, 0x4b, 0x46]).is_err());
        assert!(check_elf_magic("", &[0x7f, 0x42, 0x43, 0x46]).is_err());
    }

    #[test]
    fn parse_elf64_header_test() {
        let header_bytes = vec![
            0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x03, 0x00, 0x3e, 0x00, 0x01, 0x00, 0x00, 0x00, 0x60, 0xe1, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x1d,
            0x57, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x38, 0x00,
            0x0c, 0x00, 0x40, 0x00, 0x2c, 0x00, 0x2b, 0x00,
        ];
        let hdr_result: header::Ehdr64 = parse_elf_header(&header_bytes);

        assert_eq!(hdr_result.get_type(), header::Type::Dyn);
        assert_eq!(hdr_result.e_entry, 0xe160);
        assert_eq!(hdr_result.e_shnum, 44);
    }

    #[test]
    fn parse_elf32_header_test() {
        let header_bytes = vec![
            0x7f, 0x45, 0x4c, 0x46, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x03, 0x00, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00, 0x90, 0x10, 0x00, 0x00,
            0x34, 0x00, 0x00, 0x00, 0xe4, 0x37, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x34, 0x00,
            0x20, 0x00, 0x0c, 0x00, 0x28, 0x00, 0x1f, 0x00, 0x1e, 0x00, 0x06, 0x00, 0x34, 0x00,
            0x00, 0x00, 0x40, 0x00, 0x2c, 0x00, 0x2b, 0x00,
        ];
        let hdr_result: header::Ehdr32 = parse_elf_header(&header_bytes);

        assert_eq!(hdr_result.get_type(), header::Type::Dyn);
        assert_eq!(hdr_result.e_entry, 0x1090);
        assert_eq!(hdr_result.e_shnum, 31);
    }

    #[test]
    fn read_elf64_test() {
        let f_result = read_elf::<file::ELF64>("examples/sample");
        assert!(f_result.is_ok());
        let f = f_result.unwrap();

        assert_eq!(f.ehdr.e_entry, 0x1040);
        assert_eq!(f.ehdr.e_shnum, 29);
        assert_eq!(f.ehdr.e_shstrndx, 28);

        assert_eq!(f.sections.len(), 29);
        assert_eq!(f.segments.len(), 13);

        assert_eq!(".interp", &f.sections[1].name);
        assert_eq!(f.sections[1].header.get_type(), section::Type::ProgBits);
        assert_eq!(f.sections[1].header.sh_addr, 0x318);
        assert_eq!(f.sections[1].header.sh_offset, 0x318);
        assert_eq!(f.sections[1].header.sh_addralign, 0x1);
        assert_eq!(f.sections[1].header.sh_flags, section::SHF_ALLOC);
        assert_eq!(f.sections[1].header.sh_size, 0x1c);
        assert!(!f.sections[1].contents.clone_raw_binary().is_empty());
        assert_eq!(
            f.sections[1].contents.clone_raw_binary().len(),
            f.sections[1].header.sh_size as usize
        );

        assert_eq!(f.sections[2].header.get_type(), section::Type::Note);
        assert_eq!(f.sections[2].header.sh_addr, 0x338);
        assert!(!f.sections[2].contents.clone_raw_binary().is_empty());
        assert_eq!(
            f.sections[2].contents.clone_raw_binary().len(),
            f.sections[2].header.sh_size as usize
        );

        let rela_symbols = f.sections[10].contents.clone_rela_symbols();
        let symbols = f.sections[26].contents.clone_symbols();
        let dynamics = f.sections[21].contents.clone_dynamics();

        assert_eq!(f.sections[10].header.get_type(), section::Type::Rela);
        assert_eq!(rela_symbols.len(), 8);
        assert_eq!(f.sections[26].header.get_type(), section::Type::SymTab);
        assert_eq!(symbols.len(), 62);
        assert!(symbols[26].symbol_name.is_some());
        assert_eq!(symbols[26].symbol_name.as_ref().unwrap(), "crtstuff.c");
        assert_eq!(
            symbols[45].symbol_name.as_ref().unwrap(),
            "_ITM_deregisterTMCloneTable"
        );

        assert_eq!(f.sections[21].header.get_type(), section::Type::Dynamic);
        assert_eq!(dynamics[1].get_type(), dynamic::EntryType::Init);
        assert_eq!(dynamics[2].get_type(), dynamic::EntryType::Fini);

        assert_eq!(f.segments[0].header.get_type(), segment::Type::Phdr);
        assert_eq!(f.segments[0].header.p_flags, segment::PF_R);
        assert_eq!(f.segments[0].header.p_align, 8);

        assert_eq!(f.segments[1].header.get_type(), segment::Type::Interp);
        assert_eq!(f.segments[1].header.p_flags, segment::PF_R);
        assert_eq!(f.segments[1].header.p_align, 1);
    }
    #[test]
    fn read_elf32_test() {
        let f_result = read_elf::<file::ELF32>("examples/32bit");
        assert!(f_result.is_ok());

        let f: file::ELF32 = f_result.unwrap();

        assert_eq!(header::Type::Dyn, f.ehdr.get_type());
        assert_eq!(0x1090, f.ehdr.e_entry);
        assert_eq!(32, f.ehdr.e_phentsize);
        assert_eq!(40, f.ehdr.e_shentsize);
        assert_eq!(30, f.ehdr.e_shstrndx);

        assert_eq!(".interp", f.sections[1].name);
        assert_eq!(0x1b4, f.sections[1].header.sh_addr);
        assert_eq!(0x13, f.sections[1].header.sh_size);

        assert_eq!(".note.ABI-tag", f.sections[4].name);
        assert_eq!(0x208, f.sections[4].header.sh_addr);
    }
}
