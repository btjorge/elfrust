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
#[repr(u8)]
enum ElfVersion {
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
    fn parse(buf: &[u8]) -> Result<EIdent, &str> {
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
pub struct ElfHeader {
    pub e_ident:     [u8; EI_NIDENT],
    pub e_type:      Half,
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
pub struct ElfSectionHeader {
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

impl ElfHeader {
    fn parse(buf: &[u8]) -> Self {
        assert!(buf.len() >= size_of::<Self>());

        let ptr = buf.as_ptr() as *const Self;

        let ret : Self = unsafe {
            ptr.read_unaligned()
        };

        assert_eq!(usize::from(ret.e_phentsize), size_of::<ElfProgHeaderRaw>());
        assert_eq!(usize::from(ret.e_shentsize), size_of::<ElfSectionHeader>());

        ret
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

    fn of_word(n: Word) -> SegmentType {
        match n {
            0 => SegmentType::PT_NULL,
            1 => SegmentType::PT_LOAD,
            2 => SegmentType::PT_DYNAMIC,
            3 => SegmentType::PT_INTERP,
            4 => SegmentType::PT_NOTE,
            5 => SegmentType::PT_SHLIB,
            6 => SegmentType::PT_PHDR,
            7 => SegmentType::PT_TLS,
            Self::PT_LOOS..=Self::PT_HIOS => SegmentType::PT_OS(n),
            Self::PT_LOPROC..=Self::PT_HIPROC => SegmentType::PT_PROC(n),
            _ => panic!("invalid segment type"),
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

impl ElfProgHeader {
    pub fn of_raw(r: ElfProgHeaderRaw) -> Option<ElfProgHeader> {
        Some(
            ElfProgHeader {
                p_type: SegmentType::of_word(r.p_type),
                p_flags: SegmentFlags::from_bits(r.p_flags)?,
                p_offset: r.p_offset,
                p_vaddr:  r.p_vaddr,
                p_paddr:  r.p_paddr,
                p_filesz: r.p_filesz,
                p_memsz:  r.p_memsz,
                p_align:  r.p_align,
            }
        )
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

impl ElfSectionHeader {
    pub fn parse(buf: &[u8]) -> Self {
        assert!(buf.len() >= size_of::<Self>());

        let ptr = buf.as_ptr() as *const Self;

        unsafe { ptr.read_unaligned() }
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

        let header = ElfHeader::parse(&mmap[0..size_of::<ElfHeader>()]);

        Ok(ElfFile { ident, header, mmap })
    }

    fn get_section_from_index(&self, idx: Half) -> ElfSectionHeader {
        let off = self.header.e_shoff + u64::from(idx) * u64::from (self.header.e_shentsize);
        ElfSectionHeader::parse(self.mmap.offset(off))
    }

    fn get_section_name_table(&self) -> &[u8] {
        let idx = self.header.e_shstrndx;
        assert_ne!(idx, SHN_UNDEF);
        assert!(idx < SHN_LORESERVE);
        let hsection = self.get_section_from_index(idx);
        assert_eq!(3, hsection.sh_type); /* STRTAB */
        assert_eq!(0, hsection.sh_flags); /* none */
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
