use std::str;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct ELFHeader {
    pub e_indent: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flag: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

impl ELFHeader {
    pub const ELFMAG0: u8 = 0x7f;
    pub const ELFMAG1TO3: &str = "ELF";
    pub const SHN_LORESERVE: u16 = 0xff00;
    pub const SHN_XINDEX: u16 = 0xffff;

    pub fn get_elf_header(elf_file: &Vec<u8>) -> ELFHeader {
        let (pre, body, _) =
            unsafe { elf_file[0..std::mem::size_of::<ELFHeader>()].align_to::<ELFHeader>() };
        let header: ELFHeader = body[0].clone();
        return header;
    }

    pub fn check_is_elf(&self) -> bool {
        let magic_str = match str::from_utf8(&self.e_indent[1..4]) {
            Ok(v) => v,
            Err(e) => return false,
        };

        if self.e_indent[0] == ELFHeader::ELFMAG0 && magic_str.eq(ELFHeader::ELFMAG1TO3) {
            return true;
        }
        return false;
    }

    pub fn print(&self) {
        println!("{:?}", &self);
    }
}
