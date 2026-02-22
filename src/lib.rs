use std::fs::File;
use std::mem::offset_of;
use std::error::Error;
use strum_macros::{FromRepr, Display};
use bytemuck::{AnyBitPattern, CheckedBitPattern, NoUninit, Zeroable, cast_slice_mut, pod_read_unaligned};
use bytemuck::checked::{self, CheckedCastError};
use bytemuck::allocation::zeroed_vec;

const EI_MAG0       : usize =  0;
#[allow(unused)]
const EI_MAG1       : usize =  1;
#[allow(unused)]
const EI_MAG2       : usize =  2;
#[allow(unused)]
const EI_MAG3       : usize =  3;
const EI_CLASS      : usize =  4;
const EI_DATA       : usize =  5;
const EI_VERSION    : usize =  6;
const EI_OSABI      : usize =  7;
const EI_ABIVERSION : usize =  8;
#[allow(unused)]
const EI_PAD        : usize =  9;
const EI_NIDENT     : usize = 16;

#[derive(FromRepr, Debug, Display, PartialEq, Default, Copy, Clone, CheckedBitPattern, Zeroable)]
#[repr(u8)]
enum ElfClass {
    #[default]
    #[strum(to_string = "none")]
    ELFCLASSNONE = 0,

    #[allow(unused)]
    #[strum(to_string = "32 bits")]
    ELFCLASS32 = 1,

    #[allow(unused)]
    #[strum(to_string = "64 bits")]
    ELFCLASS64 = 2,
}

#[derive(FromRepr, Debug, Display, PartialEq, Default, Copy, Clone, CheckedBitPattern, Zeroable)]
#[repr(u8)]
enum ElfData {
    #[default]
    #[strum(to_string = "none")]
    ELFDATANONE = 0,

    #[allow(unused)]
    #[strum(to_string = "2's complement, least significant byte first")]
    ELFDATA2LSB = 1,

    #[allow(unused)]
    #[strum(to_string = "2's complement, most significant byte first")]
    ELFDATA2MSB = 2,
}

#[allow(non_camel_case_types)]
#[derive(FromRepr, Debug, Display, PartialEq, Default, Copy, Clone, CheckedBitPattern, Zeroable)]
#[repr(u8)]
pub enum ElfVersion {
    #[default]
    EV_NONE = 0,
    EV_CURRENT = 1,
}

#[derive(Debug, Clone, Copy, CheckedBitPattern, PartialEq, Zeroable)]
#[repr(C)]
pub struct EIdent {
    ei_mag:        [u8; 4],
    ei_class:      ElfClass,
    ei_data:       ElfData,
    ei_version:    ElfVersion,
    ei_osabi:      u8,
    ei_abiversion: u8,
    ei_pad :       [u8; 7],
}

const_assert_eq!(EI_NIDENT, size_of::<EIdent>());
const_assert_eq!(1, align_of::<EIdent>());
const_assert_eq!(EI_MAG0, offset_of!(EIdent, ei_mag));
const_assert_eq!(EI_CLASS, offset_of!(EIdent, ei_class));
const_assert_eq!(EI_DATA, offset_of!(EIdent, ei_data));
const_assert_eq!(EI_VERSION, offset_of!(EIdent, ei_version));
const_assert_eq!(EI_OSABI, offset_of!(EIdent, ei_osabi));
const_assert_eq!(EI_ABIVERSION, offset_of!(EIdent, ei_abiversion));
const_assert_eq!(EI_PAD, offset_of!(EIdent, ei_pad));

impl Default for EIdent {
    fn default() -> EIdent {
        EIdent { ei_mag: *b"\x7fELF", ..EIdent::zeroed()}
    }
}

use std::fmt::{self, Display, Formatter};

impl Display for EIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Elf Class: {}", self.ei_class)?;
        writeln!(f, "Data Encoding: {}", self.ei_data)?;
        writeln!(f, "Elf Version: {}", self.ei_version)?;
        writeln!(f, "OS ABI: {}", self.ei_osabi)?;
        write!(f, "ABI Version: {}", self.ei_abiversion)
    }
}

use static_assertions::{const_assert, const_assert_eq};

const_assert!(cfg!(target_endian = "little"));

impl EIdent {
    fn parse(buf: &[u8]) -> Result<EIdent, &'static str> {
        match checked::try_pod_read_unaligned::<Self>(buf) {
            Ok(ret) => {
                if &ret.ei_mag != b"\x7fELF" {
                    return Err("not an ELF file")
                }

                Ok(ret)
            },
            Err(CheckedCastError::InvalidBitPattern) => Err("invalid ELF identification"),
            Err(CheckedCastError::PodCastError(_)) => panic!("invalid cast")
        }
    }

    fn validate(&self) -> Result<(), &'static str> {
        match self.ei_version {
            ElfVersion::EV_NONE => return Err("invalid version"),
            ElfVersion::EV_CURRENT => (),
        }

        match self.ei_class {
            ElfClass::ELFCLASSNONE => return Err("invalid class"),
            ElfClass::ELFCLASS32 => return Err("unsupported class"),
            ElfClass::ELFCLASS64 => (),
        }

        match self.ei_data {
            ElfData::ELFDATANONE => return Err("invalid data encoding"),
            ElfData::ELFDATA2LSB => (),
            ElfData::ELFDATA2MSB => return Err("unsupported data encoding"),
        }

        Ok(())
    }
}

const_assert_eq!(1, size_of::<u8>());
const_assert_eq!(1, align_of::<u8>());

const_assert_eq!(8, size_of::<u64>());
const_assert_eq!(8, align_of::<u64>());

const_assert_eq!(8, size_of::<i64>());
const_assert_eq!(8, align_of::<i64>());

const_assert_eq!(2, size_of::<u16>());
const_assert_eq!(2, align_of::<u16>());

const_assert_eq!(4, size_of::<u32>());
const_assert_eq!(4, align_of::<u32>());

const_assert_eq!(4, size_of::<i32>());
const_assert_eq!(4, align_of::<i32>());

type Addr   = u64;
type Off    = u64;
type Half   = u16;
type Word   = u32;
#[expect(unused)]
type Sword  = i32;
type Xword  = u64;
#[expect(unused)]
type Sxword = i64;

const_assert!(size_of::<ElfProgHeader>() % align_of::<ElfProgHeader>() == 0);
const_assert!(size_of::<ElfSectionHeader>() % align_of::<ElfSectionHeader>() == 0);

#[derive(Clone, Copy, AnyBitPattern)]
#[repr(transparent)]
pub struct ElfType(Half);

impl ElfType {
    const ET_NONE : Half = 0;
    const ET_REL  : Half = 1;
    const ET_EXEC : Half = 2;
    const ET_DYN  : Half = 3;
    const ET_CORE : Half = 4;

    const ET_LOOS   : Half = 0xfe00;
    const ET_HIOS   : Half = 0xfeff;
    const ET_LOPROC : Half = 0xff00;
    const ET_HIPROC : Half = 0xffff;
}

impl Debug for ElfType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            Self::ET_NONE => write!(f, "ET_NONE"),
            Self::ET_REL => write!(f, "ET_REL"),
            Self::ET_EXEC => write!(f, "ET_EXEC"),
            Self::ET_DYN => write!(f, "ET_DYN"),
            Self::ET_CORE => write!(f, "ET_CORE"),
            Self::ET_LOOS..=Self::ET_HIOS => write!(f, "ET_OS({})", self.0),
            Self::ET_LOPROC..=Self::ET_HIPROC => write!(f, "ET_PROC({})", self.0),
            _ => panic!("unsupported: {}", self.0),
        }
    }
}

#[derive(Clone, Copy, CheckedBitPattern)]
#[repr(C)]
pub struct ElfHeader {
    pub e_ident:     EIdent,
    pub e_type:      ElfType,
    pub e_machine:   Half,
    pub e_version:   ElfVersion,
    pub e_entry:     Addr,
    pub e_phoff:     Off,
    pub e_shoff:     Off,
    pub e_flags:     Word,
    pub e_ehsize:    Half,
    pub e_phentsize: Half,
    pub e_phnum:     Half,
    pub e_shentsize: Half,
    pub e_shnum:     Half,
    pub e_shstrndx:  Half,
}

impl ElfHeader {
    fn parse(buf: &[u8]) -> Result<Self, &'static str> {
        assert!(buf.len() >= size_of::<Self>());

        match checked::try_pod_read_unaligned::<Self>(buf) {
            Ok(ret) => {

                assert!(ret.e_shnum < SHN_LORESERVE);
                assert!(ret.e_shstrndx < SHN_LORESERVE);

                assert_eq!(usize::from(ret.e_ehsize), size_of::<ElfHeader>());
                assert_eq!(usize::from(ret.e_phentsize), size_of::<ElfProgHeader>());
                assert_eq!(usize::from(ret.e_shentsize), size_of::<ElfSectionHeader>());

                Ok(ret)
            },
            Err(checked::CheckedCastError::InvalidBitPattern) => Err("invalid ELF version"),
            Err(checked::CheckedCastError::PodCastError(_)) => panic!("invalid cast"),
        }
    }
}

impl Display for ElfHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.e_ident)?;
        writeln!(f, "Type: {:?}", self.e_type)?;
        writeln!(f, "Machine: {}", self.e_machine)?;
        writeln!(f, "Elf Version: {}", self.e_version)?;
        writeln!(f, "Entry: {:#x}", self.e_entry)?;
        writeln!(f, "Program Header Offset: {}", self.e_phoff)?;
        writeln!(f, "Section Header Offset: {}", self.e_shoff)?;
        writeln!(f, "Flags: {}", self.e_flags)?;
        writeln!(f, "Elf Header Size: {}", self.e_ehsize)?;
        writeln!(f, "Program Header Entry Size: {}", self.e_phentsize)?;
        writeln!(f, "Program Header Number: {}", self.e_phnum)?;
        writeln!(f, "Section Header Entry Size: {}", self.e_shentsize)?;
        writeln!(f, "Section Header Number: {}", self.e_shnum)?;
        write!(f, "Section Header String Index: {}", self.e_shstrndx)
    }
}

#[derive(Clone, Copy, PartialEq, AnyBitPattern, NoUninit)]
#[repr(transparent)]
struct SegmentType(Word);

impl SegmentType {
    const PT_NULL    : Word = 0;
    const PT_LOAD    : Word = 1;
    const PT_DYNAMIC : Word = 2;
    const PT_INTERP  : Word = 3;
    const PT_NOTE    : Word = 4;
    const PT_SHLIB   : Word = 5;
    const PT_PHDR    : Word = 6;
    const PT_TLS     : Word = 7;

    const PT_LOOS    : Word = 0x60000000;
    const PT_HIOS    : Word = 0x6fffffff;
    const PT_LOPROC  : Word = 0x70000000;
    const PT_HIPROC  : Word = 0x7fffffff;
}

use fmt::Debug;

impl Debug for SegmentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            SegmentType::PT_NULL => write!(f, "PT_NULL"),
            SegmentType::PT_LOAD => write!(f, "PT_LOAD"),
            SegmentType::PT_DYNAMIC => write!(f, "PT_DYNAMIC"),
            SegmentType::PT_INTERP => write!(f, "PT_INTERP"),
            SegmentType::PT_NOTE => write!(f, "PT_NOTE"),
            SegmentType::PT_SHLIB => write!(f, "PT_SHLIB"),
            SegmentType::PT_PHDR => write!(f, "PT_PHDR"),
            SegmentType::PT_TLS => write!(f, "PT_TLS"),
            Self::PT_LOOS..=Self::PT_HIOS => write!(f, "PT_OS({})", self.0),
            Self::PT_LOPROC..=Self::PT_HIPROC => write!(f, "PT_PROC({})", self.0),
            _ => panic!("unsupported: {}", self.0),
        }
    }
}

use bitflags::{bitflags, parser};

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, AnyBitPattern, NoUninit)]
    struct SegmentFlags: Word {
        const PF_X = 0x1;
        const PF_W = 0x2;
        const PF_R = 0x4;
    }
}

const_assert_eq!(size_of::<Word>(), size_of::<SegmentFlags>());
const_assert_eq!(align_of::<Word>(), align_of::<SegmentFlags>());

impl Display for SegmentFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{{")?;
        parser::to_writer(self, &mut *f)?;
        write!(f,"}}")
    }
}

#[derive(Copy, Clone, AnyBitPattern, NoUninit)]
#[repr(C)]
pub struct ElfProgHeader {
    p_type:   SegmentType,
    p_flags:  SegmentFlags,
    p_offset: Off,
    p_vaddr:  Addr,
    p_paddr:  Addr,
    p_filesz: Xword,
    p_memsz:  Xword,
    p_align:  Xword,
}

impl ElfProgHeader {
    pub fn parse(buf: &[u8]) -> Self {
        pod_read_unaligned(buf)
    }

    #[expect(dead_code)] // TODO
    fn check(&self) -> Result<(), &str> {
        if self.p_align >= 2 &&
            (!self.p_align.is_power_of_two()
             || self.p_offset % self.p_align == self.p_vaddr % self.p_align) {
            Err("invalid alignement")
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, AnyBitPattern, NoUninit)]
#[repr(transparent)]
pub struct SectionType(Word);

impl SectionType {
    const SHT_NULL          : Word = 0;
    const SHT_PROGBITS      : Word = 1;
    const SHT_SYMTAB        : Word = 2;
    const SHT_STRTAB        : Word = 3;
    const SHT_RELA          : Word = 4;
    const SHT_HASH          : Word = 5;
    const SHT_DYNAMIC       : Word = 6;
    const SHT_NOTE          : Word = 7;
    const SHT_NOBITS        : Word = 8;
    const SHT_REL           : Word = 9;
    const SHT_SHLIB         : Word = 10;
    const SHT_DYNSYM        : Word = 11;
    const SHT_INIT_ARRAY    : Word = 14;
    const SHT_FINI_ARRAY    : Word = 15;
    const SHT_PREINIT_ARRAY : Word = 16;
    const SHT_GROUP         : Word = 17;
    const SHT_SYMTAB_SHNDX  : Word = 18;
    const SHT_RELR          : Word = 19;

    const SHT_LOOS   : Word = 0x60000000;
    const SHT_HIOS   : Word = 0x6fffffff;
    const SHT_LOPROC : Word = 0x70000000;
    const SHT_HIPROC : Word = 0x7fffffff;
    const SHT_LOUSER : Word = 0x80000000;
    const SHT_HIUSER : Word = 0xffffffff;
}

impl Debug for SectionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            Self::SHT_NULL => write!(f, "SHT_NULL"),
            Self::SHT_PROGBITS => write!(f, "SHT_PROGBITS"),
            Self::SHT_SYMTAB => write!(f, "SHT_SYMTAB"),
            Self::SHT_STRTAB => write!(f, "SHT_STRTAB"),
            Self::SHT_RELA => write!(f, "SHT_RELA"),
            Self::SHT_HASH => write!(f, "SHT_HASH"),
            Self::SHT_DYNAMIC => write!(f, "SHT_DYNAMIC"),
            Self::SHT_NOTE => write!(f, "SHT_NOTE"),
            Self::SHT_NOBITS => write!(f, "SHT_NOBITS"),
            Self::SHT_REL => write!(f, "SHT_REL"),
            Self::SHT_SHLIB => write!(f, "SHT_SHLIB"),
            Self::SHT_DYNSYM => write!(f, "SHT_DYNSYM"),
            Self::SHT_INIT_ARRAY => write!(f, "SHT_INIT_ARRAY"),
            Self::SHT_FINI_ARRAY => write!(f, "SHT_FINI_ARRAY"),
            Self::SHT_PREINIT_ARRAY => write!(f, "SHT_PREINIT_ARRAY"),
            Self::SHT_GROUP => write!(f, "SHT_GROUP"),
            Self::SHT_SYMTAB_SHNDX => write!(f, "SHT_SYMTAB_SHNDX"),
            Self::SHT_RELR => write!(f, "SHT_RELR"),
            Self::SHT_LOOS..=Self::SHT_HIOS => write!(f, "SHT_OS({})", self.0),
            Self::SHT_LOPROC..=Self::SHT_HIPROC => write!(f, "SHT_PROC({})", self.0),
            Self::SHT_LOUSER..=Self::SHT_HIUSER => write!(f, "SHT_USER({})", self.0),
            _ => panic!("unsupported: {}", self.0),
        }
    }
}

bitflags! {
    #[derive(Debug, PartialEq, Clone, Copy, AnyBitPattern, NoUninit)]
    #[repr(transparent)]
    pub struct SectionFlags: Xword {
        const SHF_WRITE            = 0x1;
        const SHF_ALLOC            = 0x2;
        const SHF_EXECINSTR        = 0x4;
        const SHF_MERGE            = 0x10;
        const SHF_STRINGS          = 0x20;
        const SHF_INFO_LINK        = 0x40;
        const SHF_LINK_ORDER       = 0x80;
        const SHF_OS_NONCONFORMING = 0x100;
        const SHF_GROUP            = 0x200;
        const SHF_TLS              = 0x400;
        const SHF_COMPRESSED       = 0x800;
    }
}

const_assert_eq!(size_of::<Xword>(), size_of::<SectionFlags>());
const_assert_eq!(align_of::<Xword>(), align_of::<SectionFlags>());

impl Display for SectionFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{{")?;
        parser::to_writer(self, &mut *f)?;
        write!(f,"}}")
    }
}

#[derive(Debug, Clone, Copy, AnyBitPattern, NoUninit)]
#[repr(C)]
pub struct ElfSectionHeader {
    pub sh_name:      Word,
    pub sh_type:      SectionType,
    pub sh_flags:     SectionFlags,
    pub sh_addr:      Addr,
    pub sh_offset:    Off,
    pub sh_size:      Xword,
    pub sh_link:      Word,
    pub sh_info:      Word,
    pub sh_addralign: Xword,
    pub sh_entsize:   Xword,
}

impl ElfSectionHeader {
    pub fn parse(buf: &[u8]) -> Self {
        assert!(buf.len() >= size_of::<Self>());

        pod_read_unaligned(buf)
    }
}

pub trait OffLen<T> {
    fn offset(&self, off: impl Into<u64>) -> &[T];
    fn offlen(&self, off: impl Into<u64>, len: impl Into<u64>) -> &[T];
}

impl OffLen<u8> for [u8] {
    fn offset(&self, off: impl Into<u64>) -> &[u8] {
        let off = usize::try_from(off.into()).unwrap();
        &self[off..]
    }

    fn offlen(&self, off: impl Into<u64>, len: impl Into<u64>) -> &[u8] {
        let len = usize::try_from(len.into()).unwrap();
        let off = usize::try_from(off.into()).unwrap();

        assert!(off <= usize::MAX - len);

        &self[off..(off+len)]
    }
}

use memmap2::Mmap;

pub struct ElfFile {
    pub ident: EIdent,
    pub header: ElfHeader,
    pub mmap: Mmap,
}

use std::io::{self, Read};
use std::path::Path;
use std::ffi::c_str::CStr;

const SHN_UNDEF : u16 = 0;
pub const SHN_LORESERVE : u16 = 0xff00;

impl ElfFile {
    pub fn open<P: AsRef<Path>>(p: P) -> Result<ElfFile, Box<dyn Error>> {
        let f = File::open(p)?;
        let mmap = unsafe { Mmap::map(&f)? };

        let ident = EIdent::parse(mmap.offlen(0u32, EI_NIDENT as u64))?;

        ident.validate()?;

        let header = ElfHeader::parse(&mmap[0..size_of::<ElfHeader>()])?;

        assert_eq!(ident, header.e_ident);

        Ok(ElfFile { ident, header, mmap })
    }

    pub fn get_program_headers(&self) -> Result<Vec<ElfProgHeader>, io::Error> {
        assert!(self.header.e_phoff != 0);
        assert!(u64::from(self.header.e_phnum) * u64::from(self.header.e_phentsize) <= 1024);
        assert_eq!(size_of::<ElfProgHeader>(), self.header.e_phentsize.into());

        let mut buf : Vec<ElfProgHeader> = zeroed_vec(self.header.e_phnum.into());

        self.mmap.offset(self.header.e_phoff).read_exact(cast_slice_mut(buf.as_mut_slice()))?;

        Ok(buf)
    }

    pub fn get_section_headers(&self) -> Result<Vec<ElfSectionHeader>, io::Error> {
        assert!(self.header.e_shoff != 0);
        assert!(u64::from(self.header.e_shnum) * u64::from(self.header.e_shentsize) <= 4096);
        assert_eq!(size_of::<ElfSectionHeader>(), self.header.e_shentsize.into());
        assert!(self.header.e_shnum != 0);
        assert!(self.header.e_shnum < SHN_LORESERVE);

        let mut buf : Vec<ElfSectionHeader> = zeroed_vec(self.header.e_shnum.into());

        self.mmap.offset(self.header.e_shoff).read_exact(cast_slice_mut(buf.as_mut_slice()))?;

        Ok(buf)
    }

    fn get_section_from_index(&self, idx: Half) -> ElfSectionHeader {
        let off = self.header.e_shoff + u64::from(idx) * u64::from (self.header.e_shentsize);
        ElfSectionHeader::parse(self.mmap.offlen(off, self.header.e_shentsize))
    }

    fn get_section_name_table(&self) -> &[u8] {
        let idx = self.header.e_shstrndx;
        assert_ne!(idx, SHN_UNDEF);
        assert!(idx < SHN_LORESERVE);
        let hsection = self.get_section_from_index(idx);
        assert_eq!(SectionType::SHT_STRTAB, hsection.sh_type.0); /* STRTAB */
        assert_eq!(SectionFlags::empty(), hsection.sh_flags); /* none */
        //assert_eq!(0, hsection.sh_addr); /* not in memory */
        assert!(hsection.sh_addralign <= 1);
        assert_eq!(0, hsection.sh_entsize); /* not fixed size entries (?) */
        assert!(hsection.sh_size <= 4096);

        self.mmap.offlen(hsection.sh_offset, hsection.sh_size)
    }

    pub fn get_section_name(&self, idx: Word) -> &CStr {
        let table = self.get_section_name_table();
        CStr::from_bytes_until_nul(table.offset(idx)).expect("invalid string section")
    }

    pub fn fmt_prog_header(&self, h: &ElfProgHeader) -> impl Display {
        struct A<'a, 'b> { file: &'a ElfFile, header: &'b ElfProgHeader }

        impl Display for A<'_, '_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let h = self.header;
                write!(f, "type:   {:?}", h.p_type)?;
                if h.p_type.0 == SegmentType::PT_INTERP {
                    let interp = CStr::from_bytes_until_nul(self.file.mmap.offset(h.p_offset)).expect("invalid interp section");
                    assert_eq!(interp.count_bytes() + 1, h.p_filesz.try_into().unwrap());
                    write!(f, " ({:?})", interp)?;
                };
                writeln!(f, "\nflags:  {}", h.p_flags)?;
                writeln!(f, "offset: {:?}", h.p_offset)?;
                writeln!(f, "vaddr:  {:?}", h.p_vaddr)?;
                writeln!(f, "paddr:  {:?}", h.p_paddr)?;
                writeln!(f, "filesz: {:?}", h.p_filesz)?;
                writeln!(f, "memsz:  {:?}", h.p_memsz)?;
                writeln!(f, "align:  {:?}", h.p_align)
            }
        }

        A { file: self, header: &h }
    }

    pub fn fmt_section_header(&self, h: &ElfSectionHeader) -> impl Display {
        struct A<'a, 'b> { file: &'a ElfFile, header: &'b ElfSectionHeader }

        impl Display for A<'_, '_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let h = self.header;

                writeln!(f, "name: {} {:?}", h.sh_name,
                         self.file.get_section_name(h.sh_name))?;
                writeln!(f, "type: {:?}", h.sh_type)?;
                writeln!(f, "flags: {}", h.sh_flags)?;
                writeln!(f, "addr: {:?}", h.sh_addr)?;
                writeln!(f, "offset: {:?}", h.sh_offset)?;
                writeln!(f, "size: {:?}", h.sh_size)?;
                writeln!(f, "link: {:?}", h.sh_link)?;
                writeln!(f, "info: {:?}", h.sh_info)?;
                writeln!(f, "addralign: {:?}", h.sh_addralign)?;
                writeln!(f, "entsize: {:?}", h.sh_entsize)
            }
        }

        A { file: self, header: h }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_eident() {
        //assert_eq!(Err("not an ELF file"),            EIdent::parse(b"\x7f"));
        assert_eq!(Err("not an ELF file"),            EIdent::parse(b"\x7fEFF\0\0\0\0\0\0\0\0\0\0\0\0"));
        assert_eq!(Err("invalid ELF identification"), EIdent::parse(b"\x7fELF\x42\0\0\0\0\0\0\0\0\0\0\0"));
        assert_eq!(Err("invalid ELF identification"), EIdent::parse(b"\x7fELF\0\x42\0\0\0\0\0\0\0\0\0\0"));
        assert_eq!(Err("invalid ELF identification"), EIdent::parse(b"\x7fELF\0\0\x42\0\0\0\0\0\0\0\0\0"));

        assert_eq!(Ok(EIdent::default()),        EIdent::parse(b"\x7fELF\0\0\0\0\0\0\0\0\0\0\0\0"));
    }

    #[test]
    fn validate_eident() {
        let default = EIdent::default();
        let current = EIdent { ei_version: ElfVersion::EV_CURRENT, ..default };
        let class64 = EIdent { ei_class: ElfClass::ELFCLASS64, ..current };
        let valid = EIdent { ei_data: ElfData::ELFDATA2LSB, ..class64};

        assert_eq!(Err("invalid version"),   default.validate());
        assert_eq!(Err("invalid class"),     current.validate());
        assert_eq!(Err("unsupported class"), EIdent { ei_class: ElfClass::ELFCLASS32, .. current }.validate());
        assert_eq!(Err("invalid data encoding"), class64.validate());
        assert_eq!(Err("unsupported data encoding"), EIdent { ei_data: ElfData::ELFDATA2MSB, ..class64}.validate());
        assert_eq!(Ok(()), valid.validate());
    }

    #[test]
    fn parse_validate_eident() {
        let eident = EIdent::parse(b"\x7fELF\x02\x01\x01\0\0\0\0\0\0\0\0\0").unwrap();
        assert_eq!(Ok(()), eident.validate());
    }
}
