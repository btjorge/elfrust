use std::fs::File;
use std::mem::offset_of;
use std::error::Error;
use std::os::unix::fs::FileExt;
use std::fmt::{self, Debug, Display, Formatter};
use strum_macros::{FromRepr, Display};
use bytemuck::{AnyBitPattern, CheckedBitPattern, Contiguous, NoUninit, Zeroable, cast_slice_mut, pod_read_unaligned};
use bytemuck::checked::{self, CheckedCastError};
use bytemuck::allocation::zeroed_vec;
use tabled::Tabled;

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
#[derive(FromRepr, Debug, Display, PartialEq, Default, Copy, Clone, CheckedBitPattern, Contiguous, Zeroable)]
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
    #[allow(dead_code)]
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
    const NONE : Half = 0;
    const REL  : Half = 1;
    const EXEC : Half = 2;
    const DYN  : Half = 3;
    const CORE : Half = 4;

    const LOOS   : Half = 0xfe00;
    const HIOS   : Half = 0xfeff;
    const LOPROC : Half = 0xff00;
    const HIPROC : Half = 0xffff;
}

impl Debug for ElfType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            Self::NONE => write!(f, "NONE"),
            Self::REL => write!(f, "REL"),
            Self::EXEC => write!(f, "EXEC"),
            Self::DYN => write!(f, "DYN"),
            Self::CORE => write!(f, "CORE"),
            Self::LOOS..=Self::HIOS => write!(f, "OS({})", self.0),
            Self::LOPROC..=Self::HIPROC => write!(f, "PROC({})", self.0),
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
    pub e_version:   Word,
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
    #[allow(dead_code)]
    fn parse(buf: &[u8]) -> Result<Self, &'static str> {
        assert!(buf.len() == size_of::<Self>());

        match checked::try_pod_read_unaligned::<Self>(buf) {
            Ok(ret) => {
                ret.e_ident.validate()?;

                assert!(ret.e_shnum < SHN_LORESERVE);
                assert!(ret.e_shstrndx < SHN_LORESERVE);

                assert_eq!(ret.e_version, ret.e_ident.ei_version.into_integer().into());

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
    const NULL    : Word = 0;
    const LOAD    : Word = 1;
    const DYNAMIC : Word = 2;
    const INTERP  : Word = 3;
    const NOTE    : Word = 4;
    const SHLIB   : Word = 5;
    const PHDR    : Word = 6;
    const TLS     : Word = 7;

    const LOOS    : Word = 0x60000000;
    const HIOS    : Word = 0x6fffffff;
    const LOPROC  : Word = 0x70000000;
    const HIPROC  : Word = 0x7fffffff;
}

impl Display for SegmentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            SegmentType::NULL => write!(f, "NULL"),
            SegmentType::LOAD => write!(f, "LOAD"),
            SegmentType::DYNAMIC => write!(f, "DYNAMIC"),
            SegmentType::INTERP => write!(f, "INTERP"),
            SegmentType::NOTE => write!(f, "NOTE"),
            SegmentType::SHLIB => write!(f, "SHLIB"),
            SegmentType::PHDR => write!(f, "PHDR"),
            SegmentType::TLS => write!(f, "TLS"),
            Self::LOOS..=Self::HIOS => write!(f, "OS({})", self.0),
            Self::LOPROC..=Self::HIPROC => write!(f, "PROC({})", self.0),
            _ => panic!("unsupported: {}", self.0),
        }
    }
}

use bitflags::{bitflags, parser};

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, AnyBitPattern, NoUninit)]
    struct SegmentFlags: Word {
        const X = 0x1;
        const W = 0x2;
        const R = 0x4;
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

#[derive(Copy, Clone, AnyBitPattern, NoUninit, Tabled)]
#[tabled(display(Addr, "display_addr"))]
#[repr(C)]
pub struct ElfProgHeader {
    #[tabled(rename = "type")]
    type_:   SegmentType,
    flags:  SegmentFlags,
    offset: Off,
    vaddr:  Addr,
    paddr:  Addr,
    filesz: Xword,
    memsz:  Xword,
    align:  Xword,
}

impl ElfProgHeader {
    pub fn parse(buf: &[u8]) -> Self {
        pod_read_unaligned(buf)
    }

    #[expect(dead_code)] // TODO
    fn check(&self) -> Result<(), &str> {
        if self.align >= 2 &&
            (!self.align.is_power_of_two()
             || self.offset % self.align == self.vaddr % self.align) {
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
    const NULL          : Word = 0;
    const PROGBITS      : Word = 1;
    const SYMTAB        : Word = 2;
    const STRTAB        : Word = 3;
    const RELA          : Word = 4;
    const HASH          : Word = 5;
    const DYNAMIC       : Word = 6;
    const NOTE          : Word = 7;
    const NOBITS        : Word = 8;
    const REL           : Word = 9;
    const SHLIB         : Word = 10;
    const DYNSYM        : Word = 11;
    const INIT_ARRAY    : Word = 14;
    const FINI_ARRAY    : Word = 15;
    const PREINIT_ARRAY : Word = 16;
    const GROUP         : Word = 17;
    const SYMTAB_SHNDX  : Word = 18;
    const RELR          : Word = 19;

    const LOOS   : Word = 0x60000000;
    const HIOS   : Word = 0x6fffffff;
    const LOPROC : Word = 0x70000000;
    const HIPROC : Word = 0x7fffffff;
    const LOUSER : Word = 0x80000000;
    const HIUSER : Word = 0xffffffff;
}

impl Display for SectionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            Self::NULL => write!(f, "NULL"),
            Self::PROGBITS => write!(f, "PROGBITS"),
            Self::SYMTAB => write!(f, "SYMTAB"),
            Self::STRTAB => write!(f, "STRTAB"),
            Self::RELA => write!(f, "RELA"),
            Self::HASH => write!(f, "HASH"),
            Self::DYNAMIC => write!(f, "DYNAMIC"),
            Self::NOTE => write!(f, "NOTE"),
            Self::NOBITS => write!(f, "NOBITS"),
            Self::REL => write!(f, "REL"),
            Self::SHLIB => write!(f, "SHLIB"),
            Self::DYNSYM => write!(f, "DYNSYM"),
            Self::INIT_ARRAY => write!(f, "INIT_ARRAY"),
            Self::FINI_ARRAY => write!(f, "FINI_ARRAY"),
            Self::PREINIT_ARRAY => write!(f, "PREINIT_ARRAY"),
            Self::GROUP => write!(f, "GROUP"),
            Self::SYMTAB_SHNDX => write!(f, "SYMTAB_SHNDX"),
            Self::RELR => write!(f, "RELR"),
            Self::LOOS..=Self::HIOS => write!(f, "OS({})", self.0),
            Self::LOPROC..=Self::HIPROC => write!(f, "PROC({})", self.0),
            Self::LOUSER..=Self::HIUSER => write!(f, "USER({})", self.0),
            _ => panic!("unsupported: {}", self.0),
        }
    }
}

impl Debug for SectionType {
    fn fmt (&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

bitflags! {
    #[derive(Debug, PartialEq, Clone, Copy, AnyBitPattern, NoUninit)]
    #[repr(transparent)]
    pub struct SectionFlags: Xword {
        const WRITE            = 0x1;
        const ALLOC            = 0x2;
        const EXECINSTR        = 0x4;
        const MERGE            = 0x10;
        const STRINGS          = 0x20;
        const INFO_LINK        = 0x40;
        const LINK_ORDER       = 0x80;
        const OS_NONCONFORMING = 0x100;
        const GROUP            = 0x200;
        const TLS              = 0x400;
        const COMPRESSED       = 0x800;
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

#[derive(Debug, Clone, Copy, AnyBitPattern, NoUninit, Tabled)]
#[tabled(display(Addr, "display_addr"))]
#[repr(C)]
pub struct ElfSectionHeader {
    pub name:      Word,
    #[tabled(rename = "type")]
    pub type_:     SectionType,
    pub flags:     SectionFlags,
    pub addr:      Addr,
    pub offset:    Off,
    pub size:      Xword,
    pub link:      Word,
    pub info:      Word,
    pub align:     Xword,
    pub entsize:   Xword,
}

impl ElfSectionHeader {
    pub fn parse(buf: &[u8]) -> Self {
        assert!(buf.len() >= size_of::<Self>());

        pod_read_unaligned(buf)
    }
}

fn display_addr(a: &Addr) -> String {
    format!("{:#x}", a)
}

const_assert_eq!(64, usize::BITS);
const_assert_eq!(64, isize::BITS);

pub struct StringTableSectionIterator<'a> {
    buf: &'a [u8],
    index: usize,
}

impl<'a> Iterator for StringTableSectionIterator<'a> {
    type Item = (Word, &'a CStr);

    fn next(&mut self) -> Option<(Word, &'a CStr)> {
        if self.index >= self.buf.len() {
            return None
        };

        let index = self.index;
        let s = CStr::from_bytes_until_nul(&self.buf[index..]).unwrap();
        self.index += s.count_bytes() + 1;
        Some((index.try_into().unwrap(), s))
    }
}

#[derive(Debug)]
pub struct StringTableSection {
    pub name : Word,
    content : Vec<u8>,
}

impl StringTableSection {
    fn new(name: Word, buf: Vec<u8>) -> Self {
        assert!(buf[0] == 0u8);
        assert!(buf[buf.len()-1] == 0u8);

        StringTableSection { name, content: buf }
    }
}

impl<'a> IntoIterator for &'a StringTableSection {
    type Item = (Word, &'a CStr);
    type IntoIter = StringTableSectionIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        StringTableSectionIterator {
            buf: self.content.as_slice(),
            index: 0,
        }
    }
}

pub struct ElfFile {
    header: ElfHeader,
    program_headers: Vec<ElfProgHeader>,
    section_headers: Vec<ElfSectionHeader>,
    section_names: Option<Vec<u8>>,
    file: File,
}

use std::io;
use std::path::Path;
use std::ffi::c_str::{CStr, CString};

const SHN_UNDEF : u16 = 0;
pub const SHN_LORESERVE : u16 = 0xff00;

#[derive(Debug)]
struct StringError<'a>(&'a str);

impl Display for StringError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for StringError<'_> {}

const SIZE: u64 = 32768;

impl ElfFile {
    pub fn read_elf_header(f: &File) -> Result<ElfHeader, Box<dyn Error>> {
        let mut buf = [0u8; size_of::<ElfHeader>()];

        f.read_exact_at(&mut buf, 0)?;

        ElfHeader::parse(&buf).map_err(|s| Box::new(StringError(s)) as Box<dyn Error>)
    }

    pub fn from_file(file: File) -> Result<ElfFile, Box<dyn Error>> {
        let header = Self::read_elf_header(&file)?;
        let program_headers = Self::read_program_headers(header, &file)?;
        let section_headers = Self::read_section_headers(header, &file)?;

        let section_names = Self::read_section_name_table(header, &section_headers[..], &file)?;

        Ok(ElfFile {
            header,
            program_headers,
            section_headers,
            section_names,
            file
        })
    }

    pub fn open<P: AsRef<Path>>(p: P) -> Result<ElfFile, Box<dyn Error>> {
        Self::from_file(File::open(p)?)
    }

    fn read_program_headers(header: ElfHeader, file: &File) -> Result<Vec<ElfProgHeader>, io::Error> {
        if header.e_phoff == 0 {
            assert_eq!(0, header.e_phnum); /* extra conformity check */
            return Ok(Vec::new())
        }

        assert!(header.e_phoff != 0);
        assert!(u64::from(header.e_phnum) * u64::from(header.e_phentsize) <= SIZE);
        assert_eq!(size_of::<ElfProgHeader>(), header.e_phentsize.into());

        let mut buf : Vec<ElfProgHeader> = zeroed_vec(header.e_phnum.into());

        file.read_exact_at(cast_slice_mut(buf.as_mut_slice()), header.e_phoff)?;

        Ok(buf)
    }

    fn read_section_headers(header: ElfHeader, file: &File) -> Result<Vec<ElfSectionHeader>, io::Error> {
        if header.e_shoff == 0 {
            assert_eq!(0, header.e_shnum); /* extra conformity check */
            return Ok(Vec::new())
        }

        assert!(header.e_shoff != 0);
        assert!(u64::from(header.e_shnum) * u64::from(header.e_shentsize) <= SIZE);
        assert_eq!(size_of::<ElfSectionHeader>(), header.e_shentsize.into());
        assert!(header.e_shnum != 0); /* valid, not supported yet */
        assert!(header.e_shnum < SHN_LORESERVE);

        let mut buf : Vec<ElfSectionHeader> = zeroed_vec(header.e_shnum.into());

        file.read_exact_at(cast_slice_mut(buf.as_mut_slice()), header.e_shoff)?;

        Ok(buf)
    }

    fn read_section_name_table(header: ElfHeader, sections: &[ElfSectionHeader], file: &File) -> io::Result<Option<Vec<u8>>> {
        let idx = header.e_shstrndx;
        assert_ne!(idx, SHN_UNDEF);
        assert!(idx < SHN_LORESERVE);
        let hsection = sections[usize::from(idx)];
        assert_eq!(SectionType::STRTAB, hsection.type_.0); /* STRTAB */
        assert_eq!(SectionFlags::empty(), hsection.flags); /* none */
        //assert_eq!(0, hsection.addr); /* not in memory */
        assert!(hsection.align <= 1);
        assert_eq!(0, hsection.entsize); /* not fixed size entries (?) */
        assert!(hsection.size <= SIZE);

        let mut buf = vec!(0u8; hsection.size.try_into().unwrap());
        file.read_exact_at(&mut buf, hsection.offset)?;
        Ok(Some(buf))
    }

    pub fn get_section_name(&self, idx: Word) -> &CStr {
        let idx : usize = idx.try_into().unwrap();
        let table = self.section_names.as_ref().unwrap();
        CStr::from_bytes_until_nul(&table[idx..]).expect("invalid string section")
    }

    pub fn fmt_prog_header(&self, h: &ElfProgHeader) -> impl Display {
        struct A<'a, 'b> { file: &'a ElfFile, header: &'b ElfProgHeader }

        impl Display for A<'_, '_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let h = self.header;
                write!(f, "type:   {}", h.type_)?;
                if h.type_.0 == SegmentType::INTERP {
                    let mut buf = vec!(0u8; h.filesz.try_into().unwrap());
                    self.file.file.read_exact_at(&mut buf, h.offset).unwrap();
                    write!(f, " ({:?})", CStr::from_bytes_with_nul(&buf).expect("invalid interp section"))?;
                };
                writeln!(f, "\nflags:  {}", h.flags)?;
                writeln!(f, "offset: {:?}", h.offset)?;
                writeln!(f, "vaddr:  {:?}", h.vaddr)?;
                writeln!(f, "paddr:  {:?}", h.paddr)?;
                writeln!(f, "filesz: {:?}", h.filesz)?;
                writeln!(f, "memsz:  {:?}", h.memsz)?;
                writeln!(f, "align:  {:?}", h.align)
            }
        }

        A { file: self, header: &h }
    }

    pub fn fmt_section_header(&self, h: &ElfSectionHeader) -> impl Display {
        struct A<'a, 'b> { file: &'a ElfFile, header: &'b ElfSectionHeader }

        impl Display for A<'_, '_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let h = self.header;

                writeln!(f, "name: {} {:?}", h.name,
                         self.file.get_section_name(h.name))?;
                writeln!(f, "type: {}", h.type_)?;
                writeln!(f, "flags: {}", h.flags)?;
                writeln!(f, "addr: {:?}", h.addr)?;
                writeln!(f, "offset: {:?}", h.offset)?;
                writeln!(f, "size: {:?}", h.size)?;
                writeln!(f, "link: {:?}", h.link)?;
                writeln!(f, "info: {:?}", h.info)?;
                writeln!(f, "addralign: {:?}", h.align)?;
                writeln!(f, "entsize: {:?}", h.entsize)
            }
        }

        A { file: self, header: h }
    }

    pub fn header (&self) -> &ElfHeader {
        &self.header
    }

    pub fn program_headers(&self) -> &[ElfProgHeader] {
        self.program_headers.as_slice()
    }

    pub fn section_headers(&self) -> &[ElfSectionHeader] {
        self.section_headers.as_slice()
    }

    pub fn interp(&self) -> io::Result<Option<CString>> {
        match self.program_headers.iter().find(|h| h.type_.0 == SegmentType::INTERP) {
            None => Ok(None),
            Some(header) => {
                let mut buf = vec!(0u8; header.filesz.try_into().unwrap());
                self.file.read_exact_at(&mut buf, header.offset)?;
                Ok(Some(CStr::from_bytes_with_nul(&buf).expect("invalid interp section").to_owned()))
            }
        }
    }

    pub fn string_table_sections(&self) -> impl Iterator<Item = StringTableSection> {
        self.section_headers.iter()
            .filter_map(|h| {
                if h.type_.0 != SectionType::STRTAB {
                    return None
                }

                assert!(h.size <= SIZE);

                let mut buf = vec![0u8; h.size.try_into().unwrap()];
                self.file.read_exact_at(&mut buf[..], h.offset).unwrap();
                Some(StringTableSection::new(h.name, buf))
            })
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
