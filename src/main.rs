#![allow(unused)]
use clap::{CommandFactory, Parser};
use rustc_hex::ToHex;
use std::io;
use std::io::Write;
use std::str;

mod elf_reader;
pub use crate::elf_reader::elf_header::ELFHeader;
pub use crate::elf_reader::section_header::SectionHeader;
pub use crate::elf_reader::section_header_table::SectionHeaderTable;



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

fn print_python_shellcode(binary: &Vec<u8>) {
    let hex_str: String = binary.as_slice().to_hex();
    print!("python3 -c 'import sys; sys.stdout.buffer.write(b\"");
    for i in 0..hex_str.len() / 2 {
        print!("\\x{}", &hex_str[i * 2..i * 2 + 2]);
    }
    println!("\")'");
}

fn stdout_shellcode(binary: &Vec<u8>) {
    let mut writer = io::BufWriter::new(io::stdout());
    writer.write_all(&binary.as_slice());
}

/// Read text section bytes and format it (64bit ELF only)
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

    /// Ex: python3 -c 'import sys; sys.stdout.buffer.write("\x55\x48\x89\xe5\x48")'
    #[clap(long, short, action)]
    python_mode: bool,

    /// Direct stdout. If you choose this option, other option will be ignore.
    #[clap(long, short, action)]
    write_mode: bool,

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

    if args.write_mode {
        stdout_shellcode(&text_section);
        return;
    }

    let mut arg_rem = args.array_mode as u8 + args.python_mode as u8 + args.string_mode as u8;

    if args.string_mode {
        print_string_shellcode(&text_section);
        arg_rem -= 1;
        if arg_rem == 0 {
            return;
        } else {
            println!();
        }
    }

    if args.array_mode {
        print_array_shellcode(&text_section);
        arg_rem -= 1;
        if arg_rem == 0 {
            return;
        } else {
            println!();
        }
    }

    if args.python_mode {
        print_python_shellcode(&text_section);
        return;
    }

    print_hex(&text_section);
    return;
}
