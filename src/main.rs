#![allow(unused)]
use clap::{CommandFactory, Parser};
use rustc_hex::ToHex;
use std::str;

#[repr(C)]
#[derive(Clone, Debug)]
struct SectionHeader {
    sh_name: u32,
    sh_type: u32,
    sh_flags: u64,
    sh_addr: u64,
    sh_offset: u64,
    sh_size: u64,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: u64,
    sh_entsize: u64,
}

#[allow(non_camel_case_types)]
enum SH_TYPE {
    SHT_NULL,
    SHT_PROGBITS,
    SHT_SYMTAB,
    SHT_STRTAB,
    SHT_RELA,
    SHT_HASH,
    SHT_DYNAMIC,
    SHT_NOTE,
    SHT_NOBITS,
    SHT_REL,
    SHT_SHLIB,
    SHT_DYNSYM,
    SHT_LOPROC,
    SHT_HIPROC,
    SHT_LOUSER,
    SHT_HIUSER,
}

#[derive(Clone, Debug)]
struct SectionHeaderTable {
    section_headers: Vec<SectionHeader>,
    section_names: Vec<String>,
}

impl SectionHeaderTable {
    fn get_section_header_table(elf_header: &ELFHeader, elf_file: &Vec<u8>) -> SectionHeaderTable {
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

    fn get_section_names(
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
    fn is_progbits_section(&self, idx: usize) -> bool {
        return SH_TYPE::SHT_PROGBITS as u32 == self.section_headers[idx].sh_type;
    }

    fn is_text_section(&self, idx: usize) -> bool {
        return self.is_progbits_section(idx) && self.section_names[idx] == ".text";
    }

    fn get_text_section_index(&self) -> Option<u16> {
        for i in 0..self.section_headers.len() {
            if self.is_text_section(i as usize) {
                return Some(i as u16);
            }
        }
        return None;
    }

    fn get_text_section_header(&self) -> Option<SectionHeader> {
        match self.get_text_section_index() {
            Some(i) => return Some(self.section_headers[i as usize].clone()),
            None => return None,
        }
    }

    fn get_text_section_offset(&self) -> Option<u64> {
        match self.get_text_section_header() {
            Some(h) => return Some(h.sh_offset),
            None => return None,
        }
    }

    fn get_text_section_size(&self) -> Option<u64> {
        match self.get_text_section_header() {
            Some(h) => return Some(h.sh_size),
            None => return None,
        }
    }

    fn print(&self) {
        println!("{:?}", &self);
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
struct ELFHeader {
    e_indent: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flag: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

impl ELFHeader {
    const ELFMAG0: u8 = 0x7f;
    const ELFMAG1TO3: &str = "ELF";
    const SHN_LORESERVE: u16 = 0xff00;
    const SHN_XINDEX: u16 = 0xffff;

    fn get_elf_header(elf_file: &Vec<u8>) -> ELFHeader {
        let (pre, body, _) =
            unsafe { elf_file[0..std::mem::size_of::<ELFHeader>()].align_to::<ELFHeader>() };
        let header: ELFHeader = body[0].clone();
        return header;
    }

    fn check_is_elf(&self) -> bool {
        let magic_str = match str::from_utf8(&self.e_indent[1..4]) {
            Ok(v) => v,
            Err(e) => return false,
        };

        if self.e_indent[0] == ELFHeader::ELFMAG0 && magic_str.eq(ELFHeader::ELFMAG1TO3) {
            return true;
        }
        return false;
    }

    fn print(&self) {
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

fn read_section(section_offset: u64, section_size: u64, elf_file: &Vec<u8>) -> Vec<u8> {
    return elf_file[section_offset as usize..section_offset as usize + section_size as usize]
        .to_vec();
}

fn print_hex(binary: &Vec<u8>) {
    let hex_str: String = binary.as_slice().to_hex();
    println!("{}", hex_str);
}

fn print_string_shellcode(binary: &Vec<u8>) {
    let hex_str: String = binary.as_slice().to_hex();
    for i in 0..hex_str.len() / 2 {
        print!("\\x{}", &hex_str[i * 2..i * 2 + 2]);
    }
    println!("");
}

fn print_array_shellcode(binary: &Vec<u8>) {
    let hex_str: String = binary.as_slice().to_hex();
    for i in 0..hex_str.len() / 2 {
        if i == 0 {
            print!("0x{}", &hex_str[i * 2..i * 2 + 2]);
        } else {
            print!(", 0x{}", &hex_str[i * 2..i * 2 + 2]);
        }
    }
    println!("");
}

/// Read text section bytes and parse it (64bit ELF only)
/// Without option, it just print text section bytes
#[derive(Debug, Parser)]
#[clap(verbatim_doc_comment)]
struct Args {
    /// Ex: 0x55, 0x48, 0x89, 0xe5, 0x48
    #[clap(long, short, action)]
    string_mode: bool,

    /// Ex: \x55\x48\x89\xe5\x48
    #[clap(long, short, action)]
    array_mode: bool,

    file: std::path::PathBuf,
}

fn main() {
    let args = Args::parse();
    let mut cmd = Args::command();
    let elf_file: Vec<u8> = match std::fs::read(&args.file) {
        Ok(r) => r,
        Err(e) => {
            println!("Could not open file.\n{}", e);
            std::process::exit(0);
        }
    };
    let elf_header: ELFHeader = ELFHeader::get_elf_header(&elf_file);

    if !elf_header.check_is_elf() {
        println!("Not elf file\n");
        cmd.print_help();
    }

    let section_header_table: SectionHeaderTable =
        SectionHeaderTable::get_section_header_table(&elf_header, &elf_file);

    let text_section_offset = match section_header_table.get_text_section_offset() {
        Some(i) => i,
        None => {
            println!("Could not read section header. Abnormal ELF file");
            std::process::exit(0);
        }
    };

    let text_section_size = match section_header_table.get_text_section_size() {
        Some(i) => i,
        None => {
            println!("Could not read section header. Abnormal ELF file");
            std::process::exit(0);
        }
    };

    let text_section = read_section(text_section_offset, text_section_size, &elf_file);

    if args.string_mode && args.array_mode {
        print_string_shellcode(&text_section);
        println!("");
        print_array_shellcode(&text_section);
        return;
    }

    if args.string_mode {
        print_string_shellcode(&text_section);
        return;
    }

    if args.array_mode {
        print_array_shellcode(&text_section);
        return;
    }

    print_hex(&text_section);
    return;
}
