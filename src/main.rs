use std::fs::File;
use std::io::{self, Read};
use std::env;
use std::error::Error;
use strum_macros::{FromRepr, Display};

const EI_MAG0  : usize = 0;
const EI_MAG1  : usize = 1;
const EI_MAG2  : usize = 2;
const EI_MAG3  : usize = 3;
const EI_CLASS : usize = 4;

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

struct EIdent {
    class : ElfClass
}

fn parse_e_ident<T: Read>(f : &mut T) -> Result<EIdent, io::Error> {
    let mut buf = [0; 16];

    f.read_exact(&mut buf)?;

    if buf[EI_MAG0] != 0x7f
        && buf[EI_MAG1] != b'E'
        && buf[EI_MAG2] != b'L'
        && buf[EI_MAG3] != b'F'
    {
        panic!("not an ELF file");
    }

    Ok(EIdent {
        class: ElfClass::from_repr(buf[EI_CLASS]).expect("invalid class")
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("bad number of arguments");
    }

    let mut f = File::open(&args[1])?;

    let id = parse_e_ident(&mut f)?;

    println!("Elf Class: {}", id.class);

    Ok(())
}
