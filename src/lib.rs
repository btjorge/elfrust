use std::fs::File;
use std::error::Error;
use strum_macros::{FromRepr, Display};

const EI_MAG0       : usize =  0;
const EI_MAG1       : usize =  1;
const EI_MAG2       : usize =  2;
const EI_MAG3       : usize =  3;
const EI_CLASS      : usize =  4;
const EI_DATA       : usize =  5;
const EI_VERSION    : usize =  6;
const EI_OSABI      : usize =  7;
const EI_ABIVERSION : usize =  8;
#[allow(unused)]
const EI_PAD        : usize =  9;
const EI_NIDENT     : usize = 16;

#[derive(FromRepr, Debug, Display, PartialEq, Default, Copy, Clone)]
#[repr(u8)]
enum ElfClass {
    #[default]
    #[strum(to_string = "none")]
    ELFCLASSNONE = 0,

    #[strum(to_string = "32 bits")]
    ELFCLASS32 = 1,

    #[strum(to_string = "64 bits")]
    ELFCLASS64 = 2,
}

#[derive(FromRepr, Debug, Display, PartialEq, Default, Copy, Clone)]
#[repr(u8)]
enum ElfData {
    #[default]
    #[strum(to_string = "none")]
    ELFDATANONE = 0,

    #[strum(to_string = "2's complement, least significant byte first")]
    ELFDATA2LSB = 1,

    #[strum(to_string = "2's complement, most significant byte first")]
    ELFDATA2MSB = 2,
}

#[allow(non_camel_case_types)]
#[derive(FromRepr, Debug, Display, PartialEq, Default, Copy, Clone)]
#[repr(u32)] // ??
pub enum ElfVersion {
    #[default]
    EV_NONE = 0,
    EV_CURRENT = 1,
}

#[derive(Debug, PartialEq, Default, Copy, Clone)]
pub struct EIdent {
    class : ElfClass,
    data : ElfData,
    version : ElfVersion,
    osabi : u8,
    abiversion : u8,
}

use std::fmt::{self, Display};

impl Display for EIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Elf Class: {}", self.class)?;
        writeln!(f, "Data Encoding: {}", self.data)?;
        writeln!(f, "Elf Version: {}", self.version)?;
        writeln!(f, "OS ABI: {}", self.osabi)?;
        write!(f, "ABI Version: {}", self.abiversion)
    }
}

use static_assertions::{const_assert, const_assert_eq};

const_assert!(cfg!(target_endian = "little"));

impl EIdent {
    fn parse(buf: &[u8]) -> Result<EIdent, &'static str> {
        if buf.len() < EI_NIDENT
            || buf[EI_MAG0] != 0x7f
            || buf[EI_MAG1] != b'E'
            || buf[EI_MAG2] != b'L'
            || buf[EI_MAG3] != b'F'
        {
            return Err("not an ELF file");
        }

        let class = ElfClass::from_repr(buf[EI_CLASS]).ok_or("invalid class")?;
        let data = ElfData::from_repr(buf[EI_DATA]).ok_or("invalid data encoding")?;
        let version = ElfVersion::from_repr(buf[EI_VERSION].into()).ok_or("invalid version")?;
        let osabi = buf[EI_OSABI];
        let abiversion = buf[EI_ABIVERSION];

        Ok(EIdent {
            class: class,
            data: data,
            version: version,
            osabi: osabi,
            abiversion: abiversion,
        })
    }

    fn validate(&self) -> Result<(), &str> {
        match self.version {
            ElfVersion::EV_NONE => return Err("invalid version"),
            ElfVersion::EV_CURRENT => (),
        }

        match self.class {
            ElfClass::ELFCLASSNONE => return Err("invalid class"),
            ElfClass::ELFCLASS32 => return Err("unsupported class"),
            ElfClass::ELFCLASS64 => (),
        }

        match self.data {
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

#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct ElfHeaderRaw {
    e_ident:     [u8; EI_NIDENT],
    e_type:      Half,
    e_machine:   Half,
    e_version:   Word,
    e_entry:     Addr,
    e_phoff:     Off,
    e_shoff:     Off,
    e_flags:     Word,
    e_ehsize:    Half,
    e_phentsize: Half,
    e_phnum:     Half,
    e_shentsize: Half,
    e_shnum:     Half,
    e_shstrndx:  Half,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ElfProgHeaderRaw {
    pub p_type:   Word,
    pub p_flags:  Word,
    pub p_offset: Off,
    pub p_vaddr:  Addr,
    pub p_paddr:  Addr,
    pub p_filesz: Xword,
    pub p_memsz:  Xword,
    pub p_align:  Xword,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ElfSectionHeaderRaw {
    pub sh_name:      Word,
    pub sh_type:      Word,
    pub sh_flags:     Xword,
    pub sh_addr:      Addr,
    pub sh_offset:    Off,
    pub sh_size:      Xword,
    pub sh_link:      Word,
    pub sh_info:      Word,
    pub sh_addralign: Xword,
    pub sh_entsize:   Xword,
}

const_assert!(size_of::<ElfProgHeaderRaw>() % align_of::<ElfProgHeaderRaw>() == 0);

impl ElfHeaderRaw {
    fn parse(buf: &[u8]) -> Self {
        assert!(buf.len() >= size_of::<Self>());

        let ptr = buf.as_ptr() as *const Self;

        let ret : Self = unsafe {
            ptr.read_unaligned()
        };

        assert_eq!(usize::from(ret.e_ehsize), size_of::<ElfHeaderRaw>());
        assert_eq!(usize::from(ret.e_phentsize), size_of::<ElfProgHeaderRaw>());
        assert_eq!(usize::from(ret.e_shentsize), size_of::<ElfSectionHeaderRaw>());

        ret
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum ElfType {
    ET_NONE,
    ET_REL,
    ET_EXEC,
    ET_DYN,
    ET_CORE,
    ET_OS(Half),
    ET_PROC(Half),
}

impl ElfType {
    pub const ET_LOOS   : Half = 0xfe00;
    pub const ET_HIOS   : Half = 0xfeff;
    pub const ET_LOPROC : Half = 0xff00;
    pub const ET_HIPROC : Half = 0xffff;
}

impl TryFrom<Half> for ElfType {
    type Error = &'static str;

    fn try_from(n: Half) -> Result<Self, Self::Error> {
        match n {
            0 => Ok(ElfType::ET_NONE),
            1 => Ok(ElfType::ET_REL),
            2 => Ok(ElfType::ET_EXEC),
            3 => Ok(ElfType::ET_DYN),
            4 => Ok(ElfType::ET_CORE),
            ElfType::ET_LOOS..=ElfType::ET_HIOS => Ok(ElfType::ET_OS(n)),
            ElfType::ET_LOPROC..=ElfType::ET_HIPROC => Ok(ElfType::ET_PROC(n)),
            _ => Err("invalid ELF type")
        }
    }
}

#[derive(Debug)]
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

impl TryFrom<ElfHeaderRaw> for ElfHeader {
    type Error = &'static str;

    fn try_from(r: ElfHeaderRaw) -> Result<Self, &'static str> {
        let e_ident = EIdent::parse(&r.e_ident)?;
        let e_type = ElfType::try_from(r.e_type)?;
        let e_version = ElfVersion::from_repr(r.e_version).ok_or("invalid version")?;

        assert!(r.e_shnum < SHN_LORESERVE);
        assert!(r.e_shstrndx < SHN_LORESERVE); /* TODO */

        Ok(
            ElfHeader {
                e_ident,
                e_type,
                e_machine: r.e_machine,
                e_version,
                e_entry: r.e_entry,
                e_phoff: r.e_phoff,
                e_shoff: r.e_shoff,
                e_flags: r.e_flags,
                e_ehsize: r.e_ehsize,
                e_phentsize: r.e_phentsize,
                e_phnum: r.e_phnum,
                e_shentsize: r.e_shentsize,
                e_shnum: r.e_shnum,
                e_shstrndx: r.e_shstrndx,
            }
        )
    }
}

impl Display for ElfHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Elf Class: {}", self.e_ident.class)?;
        writeln!(f, "Data Encoding: {}", self.e_ident.data)?;
        writeln!(f, "Elf Version: {}", self.e_ident.version)?;
        writeln!(f, "OS ABI: {}", self.e_ident.osabi)?;
        writeln!(f, "ABI Version: {}", self.e_ident.abiversion)?;
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

impl ElfProgHeaderRaw {
    pub fn parse(buf: &[u8]) -> Self {
        assert!(buf.len() >= size_of::<Self>());

        let ptr = buf.as_ptr() as *const Self;

        unsafe { ptr.read_unaligned() }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
enum SegmentType {
    PT_NULL,
    PT_LOAD,
    PT_DYNAMIC,
    PT_INTERP,
    PT_NOTE,
    PT_SHLIB,
    PT_PHDR,
    PT_TLS,
    PT_OS(Word),
    PT_PROC(Word),
}

impl SegmentType {
    const PT_LOOS   : Word = 0x60000000;
    const PT_HIOS   : Word = 0x6fffffff;
    const PT_LOPROC : Word = 0x70000000;
    const PT_HIPROC : Word = 0x7fffffff;
}

impl TryFrom<Word> for SegmentType {
    type Error = &'static str;

    fn try_from(n: Word) -> Result<SegmentType, &'static str> {
        match n {
            0 => Ok(SegmentType::PT_NULL),
            1 => Ok(SegmentType::PT_LOAD),
            2 => Ok(SegmentType::PT_DYNAMIC),
            3 => Ok(SegmentType::PT_INTERP),
            4 => Ok(SegmentType::PT_NOTE),
            5 => Ok(SegmentType::PT_SHLIB),
            6 => Ok(SegmentType::PT_PHDR),
            7 => Ok(SegmentType::PT_TLS),
            Self::PT_LOOS..=Self::PT_HIOS => Ok(SegmentType::PT_OS(n)),
            Self::PT_LOPROC..=Self::PT_HIPROC => Ok(SegmentType::PT_PROC(n)),
            _ => Err("invalid segment type"),
        }
    }
}

use bitflags::{bitflags, parser};

bitflags! {
    struct SegmentFlags: Word {
        const PF_X = 0x1;
        const PF_W = 0x2;
        const PF_R = 0x4;
    }
}

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

impl TryFrom<ElfProgHeaderRaw> for ElfProgHeader {
    type Error = &'static str;

    fn try_from(r: ElfProgHeaderRaw) -> Result<ElfProgHeader, &'static str> {
        Ok(
            ElfProgHeader {
                p_type: SegmentType::try_from(r.p_type)?,
                p_flags: SegmentFlags::from_bits_retain(r.p_flags),
                p_offset: r.p_offset,
                p_vaddr:  r.p_vaddr,
                p_paddr:  r.p_paddr,
                p_filesz: r.p_filesz,
                p_memsz:  r.p_memsz,
                p_align:  r.p_align,
            }
        )
    }
}

impl ElfProgHeader {
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

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum SectionType {
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
    SHT_INIT_ARRAY,
    SHT_FINI_ARRAY,
    SHT_PREINIT_ARRAY,
    SHT_GROUP,
    SHT_SYMTAB_SHNDX,
    SHT_RELR,
    SHT_OS(Word),
    SHT_PROC(Word),
    SHT_USER(Word),
}

impl SectionType {
    const SHT_LOOS   : Word = 0x60000000;
    const SHT_HIOS   : Word = 0x6fffffff;
    const SHT_LOPROC : Word = 0x70000000;
    const SHT_HIPROC : Word = 0x7fffffff;
    const SHT_LOUSER : Word = 0x80000000;
    const SHT_HIUSER : Word = 0xffffffff;
}

impl TryFrom<Word> for SectionType {
    type Error = &'static str;

    fn try_from(n: Word) -> Result<Self, Self::Error> {
        match n {
            0 => Ok(SectionType::SHT_NULL),
            1 => Ok(SectionType::SHT_PROGBITS),
            2 => Ok(SectionType::SHT_SYMTAB),
            3 => Ok(SectionType::SHT_STRTAB),
            4 => Ok(SectionType::SHT_RELA),
            5 => Ok(SectionType::SHT_HASH),
            6 => Ok(SectionType::SHT_DYNAMIC),
            7 => Ok(SectionType::SHT_NOTE),
            8 => Ok(SectionType::SHT_NOBITS),
            9 => Ok(SectionType::SHT_REL),
            10 => Ok(SectionType::SHT_SHLIB),
            11 => Ok(SectionType::SHT_DYNSYM),
            14 => Ok(SectionType::SHT_INIT_ARRAY),
            15 => Ok(SectionType::SHT_FINI_ARRAY),
            16 => Ok(SectionType::SHT_PREINIT_ARRAY),
            17 => Ok(SectionType::SHT_GROUP),
            18 => Ok(SectionType::SHT_SYMTAB_SHNDX),
            19 => Ok(SectionType::SHT_RELR),
            Self::SHT_LOOS..=Self::SHT_HIOS => Ok(Self::SHT_OS(n)),
            Self::SHT_LOPROC..=Self::SHT_HIPROC => Ok(Self::SHT_PROC(n)),
            Self::SHT_LOUSER..=Self::SHT_HIUSER => Ok(Self::SHT_USER(n)),
            _ => Err("invalid section type"),
        }
    }
}

bitflags! {
    #[derive(Debug, PartialEq)]
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

#[derive(Debug)]
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

impl ElfSectionHeaderRaw {
    pub fn parse(buf: &[u8]) -> Self {
        assert!(buf.len() >= size_of::<Self>());

        let ptr = buf.as_ptr() as *const Self;

        unsafe { ptr.read_unaligned() }
    }
}

impl TryFrom<ElfSectionHeaderRaw> for ElfSectionHeader {
    type Error =  &'static str;

    fn try_from(r: ElfSectionHeaderRaw) -> Result<Self, Self::Error> {
        let sh_type = SectionType::try_from(r.sh_type)?;
        let sh_flags = SectionFlags::from_bits_retain(r.sh_flags);

        Ok(
            ElfSectionHeader {
                sh_name: r.sh_name,
                sh_type,
                sh_flags,
                sh_addr: r.sh_addr,
                sh_offset: r.sh_offset,
                sh_size: r.sh_size,
                sh_link: r.sh_link,
                sh_info: r.sh_info,
                sh_addralign: r.sh_addralign,
                sh_entsize: r.sh_entsize,
            }
        )
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

use std::path::Path;
use std::ffi::c_str::CStr;

const SHN_UNDEF : u16 = 0;
pub const SHN_LORESERVE : u16 = 0xff00;

impl ElfFile {
    pub fn open<P: AsRef<Path>>(p: P) -> Result<ElfFile, Box<dyn Error>> {
        let f = File::open(p)?;
        let mmap = unsafe { Mmap::map(&f)? };

        let ident = EIdent::parse(&mmap)?;

        ident.validate()?;

        let rheader = ElfHeaderRaw::parse(&mmap[0..size_of::<ElfHeaderRaw>()]);
        let header = ElfHeader::try_from(rheader).unwrap();

        Ok(ElfFile { ident, header, mmap })
    }

    fn get_section_from_index(&self, idx: Half) -> Result<ElfSectionHeader, &'static str> {
        let off = self.header.e_shoff + u64::from(idx) * u64::from (self.header.e_shentsize);
        ElfSectionHeader::try_from(ElfSectionHeaderRaw::parse(self.mmap.offset(off)))
    }

    fn get_section_name_table(&self) -> &[u8] {
        let idx = self.header.e_shstrndx;
        assert_ne!(idx, SHN_UNDEF);
        assert!(idx < SHN_LORESERVE);
        let hsection = self.get_section_from_index(idx).unwrap();
        assert_eq!(SectionType::SHT_STRTAB, hsection.sh_type); /* STRTAB */
        assert_eq!(SectionFlags::empty(), hsection.sh_flags); /* none */
        assert_eq!(0, hsection.sh_addr); /* not in memory */
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
                if h.p_type == SegmentType::PT_INTERP {
                    let interp = CStr::from_bytes_until_nul(self.file.mmap.offset(h.p_offset)).expect("invalid interp section");
                    assert_eq!(interp.count_bytes() + 1, h.p_filesz.try_into().unwrap());
                    write!(f, " ({:?})", interp)?;
                };
                write!(f, "\nflags:  ")?;
                parser::to_writer(&h.p_flags, &mut *f)?;
                writeln!(f, "")?;
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
                write!(f, "flags: ")?;
                parser::to_writer(&h.sh_flags, &mut *f)?;
                writeln!(f, "")?;
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
        assert_eq!(Err("not an ELF file"),       EIdent::parse(b"\x7f"));
        assert_eq!(Err("not an ELF file"),       EIdent::parse(b"\x7fEFF\0\0\0\0\0\0\0\0\0\0\0\0"));
        assert_eq!(Err("invalid class"),         EIdent::parse(b"\x7fELF\x42\0\0\0\0\0\0\0\0\0\0\0"));
        assert_eq!(Err("invalid data encoding"), EIdent::parse(b"\x7fELF\0\x42\0\0\0\0\0\0\0\0\0\0"));
        assert_eq!(Err("invalid version"),       EIdent::parse(b"\x7fELF\0\0\x42\0\0\0\0\0\0\0\0\0"));

        assert_eq!(Ok(EIdent::default()),        EIdent::parse(b"\x7fELF\0\0\0\0\0\0\0\0\0\0\0\0"));
    }

    #[test]
    fn validate_eident() {
        let default = EIdent::default();
        let current = EIdent { version: ElfVersion::EV_CURRENT, ..default };
        let class64 = EIdent { class: ElfClass::ELFCLASS64, ..current };
        let valid = EIdent { data: ElfData::ELFDATA2LSB, ..class64};

        assert_eq!(Err("invalid version"),   default.validate());
        assert_eq!(Err("invalid class"),     current.validate());
        assert_eq!(Err("unsupported class"), EIdent { class: ElfClass::ELFCLASS32, .. current }.validate());
        assert_eq!(Err("invalid data encoding"), class64.validate());
        assert_eq!(Err("unsupported data encoding"), EIdent { data: ElfData::ELFDATA2MSB, ..class64}.validate());
        assert_eq!(Ok(()), valid.validate());
    }

    #[test]
    fn parse_validate_eident() {
        let eident = EIdent::parse(b"\x7fELF\x02\x01\x01\0\0\0\0\0\0\0\0\0").unwrap();
        assert_eq!(Ok(()), eident.validate());
    }
}
