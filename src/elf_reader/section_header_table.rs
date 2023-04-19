use crate::elf_reader::elf_header::ELFHeader;
use crate::elf_reader::section_header::SectionHeader;
use crate::elf_reader::section_header::SH_TYPE;
use std::str;

#[derive(Clone, Debug)]
pub struct SectionHeaderTable {
    pub section_headers: Vec<SectionHeader>,
    pub section_names: Vec<String>,
}

impl SectionHeaderTable {
    pub fn get_section_header_table(elf_header: &ELFHeader, elf_file: &Vec<u8>) -> SectionHeaderTable {
        let start = elf_header.e_shoff as usize;
        let end = (elf_header.e_shoff as usize
            + elf_header.e_shnum as usize * elf_header.e_shentsize as usize);
        let (_, body, _) = unsafe { elf_file[start..end].align_to::<SectionHeader>() };
        let mut section_headers: Vec<SectionHeader> = Vec::new();
        for b in body {
            section_headers.push(b.clone());
        }
        let section_names: Vec<String> =
            SectionHeaderTable::get_section_names(&elf_header, &section_headers, &elf_file);

        let section_header_table = SectionHeaderTable {
            section_headers,
            section_names,
        };

        return section_header_table;
    }

    pub fn get_section_names(
        elf_header: &ELFHeader,
        section_headers: &Vec<SectionHeader>,
        elf_file: &Vec<u8>,
    ) -> Vec<String> {
        let mut shstrt_addr: u64;
        let mut header_names: Vec<String> = Vec::new();

        if elf_header.e_shstrndx == 0 {
            return header_names;
        }

        if elf_header.e_shstrndx == ELFHeader::SHN_XINDEX {
            shstrt_addr = section_headers[0].sh_link as u64;
        } else {
            shstrt_addr = section_headers[elf_header.e_shstrndx as usize].sh_offset;
        }

        for s in section_headers {
            let str_addr = shstrt_addr + s.sh_name as u64;
            let str_len = str_len_from_bytes(&elf_file[str_addr as usize..]);
            header_names.push(
                match str::from_utf8(&elf_file[str_addr as usize..str_addr as usize + str_len]) {
                    Ok(s) => s,
                    Err(e) => {
                        println!("Abnormal ELF file.");
                        std::process::exit(0);
                    }
                }
                .to_string(),
            );
        }

        return header_names;
    }
    pub fn is_progbits_section(&self, idx: usize) -> bool {
        return SH_TYPE::SHT_PROGBITS as u32 == self.section_headers[idx].sh_type;
    }

    pub fn is_text_section(&self, idx: usize) -> bool {
        return self.is_progbits_section(idx) && self.section_names[idx] == ".text";
    }

    pub fn get_text_section_index(&self) -> Option<u16> {
        for i in 0..self.section_headers.len() {
            if self.is_text_section(i as usize) {
                return Some(i as u16);
            }
        }
        return None;
    }

    pub fn get_text_section_header(&self) -> Option<SectionHeader> {
        match self.get_text_section_index() {
            Some(i) => return Some(self.section_headers[i as usize].clone()),
            None => return None,
        }
    }

    pub fn get_text_section_offset(&self) -> Option<u64> {
        match self.get_text_section_header() {
            Some(h) => return Some(h.sh_offset),
            None => return None,
        }
    }

    pub fn get_text_section_size(&self) -> Option<u64> {
        match self.get_text_section_header() {
            Some(h) => return Some(h.sh_size),
            None => return None,
        }
    }

    pub fn print(&self) {
        println!("{:?}", &self);
    }
}

fn str_len_from_bytes(bytes_array: &[u8]) -> usize {
    let mut count: usize = 0;

    loop {
        if bytes_array[count] == b'\x00' {
            break;
        }
        count += 1;
    }

    return count;
}