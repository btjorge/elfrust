use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::env;
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

#[derive(FromRepr, Debug, Display)]
#[repr(u8)]
enum ElfClass {
    #[strum(to_string = "none")]
    ELFCLASSNONE = 0,

    #[strum(to_string = "32 bits")]
    ELFCLASS32 = 1,

    #[strum(to_string = "64 bits")]
    ELFCLASS64 = 2,
}

#[derive(FromRepr, Debug, Display)]
#[repr(u8)]
enum ElfData {
    #[strum(to_string = "none")]
    ELFDATANONE = 0,

    #[strum(to_string = "2's complement, least significant byte first")]
    ELFDATA2LSB = 1,

    #[strum(to_string = "2's complement, most significant byte first")]
    ELFDATA2MSB = 2,
}

#[allow(non_camel_case_types)]
#[derive(FromRepr, Debug, Display, PartialEq)]
enum ElfVersion {
    EV_NONE = 0,
    EV_CURRENT = 1,
}

struct EIdent {
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

impl EIdent {
    fn parse(buf: &[u8]) -> Result<EIdent, &str> {
        if buf[EI_MAG0] != 0x7f
            && buf[EI_MAG1] != b'E'
            && buf[EI_MAG2] != b'L'
            && buf[EI_MAG3] != b'F'
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
        if self.version != ElfVersion::EV_CURRENT {
            return Err("Invalid version");
        }

        match self.class {
            ElfClass::ELFCLASSNONE => return Err("Invalid class"),
            ElfClass::ELFCLASS32 => return Err("Unsupported class"),
            ElfClass::ELFCLASS64 => ()
        }

        match self.data {
            ElfData::ELFDATANONE => return Err("Invalid data encoding"),
            ElfData::ELFDATA2LSB => (),
            ElfData::ELFDATA2MSB => return Err("Unsupported data encoding")
        }

        Ok(())
    }
}

use static_assertions::{const_assert, const_assert_eq};

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

#[derive(Debug)]
#[repr(C)]
struct ElfHeader {
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

#[derive(Debug)]
#[repr(C)]
struct ElfProgHeader {
    p_type:   Word,
    p_flags:  Word,
    p_offset: Off,
    p_vaddr:  Addr,
    p_paddr:  Addr,
    p_filesz: Xword,
    p_memsz:  Xword,
    p_align:  Xword,
}

#[derive(Debug)]
#[repr(C)]
struct ElfSectionHeader {
    sh_name:      Word,
    sh_type:      Word,
    sh_flags:     Xword,
    sh_addr:      Addr,
    sh_offset:    Off,
    sh_size:      Xword,
    sh_link:      Word,
    sh_info:      Word,
    sh_addralign: Xword,
    sh_entsize:   Xword,
}

const_assert!(size_of::<ElfProgHeader>() % align_of::<ElfProgHeader>() == 0);

impl ElfHeader {
    fn parse(buf: &[u8; size_of::<Self>()]) -> Self {
        let ret : Self = unsafe {
            std::mem::transmute_copy(buf)
        };

        assert_eq!(usize::from(ret.e_phentsize), size_of::<ElfProgHeader>());
        assert_eq!(usize::from(ret.e_shentsize), size_of::<ElfSectionHeader>());

        ret
    }
}

impl ElfProgHeader {
    fn parse(buf: &[u8; size_of::<Self>()]) -> Self {
        unsafe {
            std::mem::transmute_copy(buf)
        }
    }
}

struct ElfFile {
    ident: EIdent,
    header: ElfHeader,
    file: File,
}

use std::path::Path;

impl ElfFile {
    fn open<P: AsRef<Path>>(p: P) -> Result<ElfFile, Box<dyn Error>> {
        let f = File::open(p)?;

        let mut identbuf = [0; EI_NIDENT];

        f.read_exact_at(&mut identbuf, 0)?;

        let ident = EIdent::parse(&identbuf)?;

        ident.validate()?;

        let mut headerbuf = [0; size_of::<ElfHeader>()];

        f.read_exact_at(&mut headerbuf, 0)?;

        let header = ElfHeader::parse(&headerbuf);

        Ok(ElfFile { ident, header, file: f })
    }
}

impl ElfSectionHeader {
    fn parse(buf: &[u8; size_of::<Self>()]) -> Self {
        unsafe {
            std::mem::transmute_copy(buf)
        }
    }

    fn get_from_index(f: &ElfFile, idx: Half) -> io::Result<Self> {
        let mut buf = [0; size_of::<Self>()];

        let off = f.header.e_shoff + u64::from(idx) * u64::from (f.header.e_shentsize);
        f.file.read_exact_at(&mut buf, off)?;

        Ok(ElfSectionHeader::parse(&buf))
    }
}

const SHN_UNDEF : u16 = 0;
const SHN_LORESERVE : u16 = 0xff00;

struct StringTable {
    content: Box<[u8]>,
}

use std::os::unix::fs::FileExt;
use std::ffi::c_str::CStr;

impl StringTable {
    fn load(f: &ElfFile) -> io::Result<StringTable> {
        let idx = f.header.e_shstrndx;
        assert_ne!(idx, SHN_UNDEF);
        assert!(idx < SHN_LORESERVE);
        let hsection = ElfSectionHeader::get_from_index(f, idx)?;
        assert_eq!(3, hsection.sh_type); /* STRTAB */
        assert_eq!(0, hsection.sh_flags); /* none */
        assert_eq!(0, hsection.sh_addr); /* not in memory */
        assert!(hsection.sh_addralign <= 1);
        assert_eq!(0, hsection.sh_entsize); /* not fixed size entries (?) */
        assert!(hsection.sh_size <= 4096);

        let mut buf = vec![0; hsection.sh_size.try_into().unwrap()].into_boxed_slice();

        f.file.read_exact_at(buf.as_mut(), hsection.sh_offset)?;

        Ok(StringTable { content: buf, })
    }

    fn get_string(&self, idx: Word) -> &CStr {
        CStr::from_bytes_until_nul(&self.content[idx.try_into().unwrap()..]).expect("invalid string section")
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("bad number of arguments");
    }

    let mut f = ElfFile::open(&args[1])?;

    println!("{}", f.ident);

    println!("{:?}", f.header);

    if f.header.e_phoff != 0 {
        assert_ne!(0, f.header.e_phnum);

        f.file.seek(SeekFrom::Start(f.header.e_phoff))?;

        let mut buf = [0; size_of::<ElfProgHeader>()];

        /* good if the right align-size equation holds (see above) */
        for _ in 0..f.header.e_phnum {
            f.file.read_exact(&mut buf)?;
            let pheader = ElfProgHeader::parse(&buf);
            println!("{:?}", pheader)
        }
    } else {
        assert_eq!(0, f.header.e_phnum); /* extra conformity check */
    }

    let string_table = StringTable::load(&f)?;

    if f.header.e_shoff != 0 {
        assert_ne!(0, f.header.e_shnum); /* TODO: valid, not supported yet */
        assert!(f.header.e_shnum < SHN_LORESERVE); /* extra? conformity check */

        f.file.seek(SeekFrom::Start(f.header.e_shoff))?;

        let mut buf = [0; size_of::<ElfSectionHeader>()];

        for i in 0..f.header.e_shnum {
            f.file.read_exact(&mut buf)?;
            let sheader = ElfSectionHeader::parse(&buf);
            println!("{}: {:?} {:?}", i, string_table.get_string(sheader.sh_name), sheader)
        }

    } else {
        assert_eq!(0, f.header.e_shnum); /* extra conformity check */
    }

    Ok(())
}
