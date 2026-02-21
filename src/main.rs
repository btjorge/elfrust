use std::env;
use std::error::Error;
use elfrust::{self, ElfFile, ElfProgHeader, ElfSectionHeader, OffLen};

fn print_segments (f: &ElfFile) {
    if f.header.e_phoff != 0 {
        assert_ne!(0, f.header.e_phnum);

        /* good if the right align-size equation holds (see above) */
        for i in 0..f.header.e_phnum {
            let offset = f.header.e_phoff + u64::from(i) * u64::from(f.header.e_phentsize);
            let slice = f.mmap.offlen(offset, f.header.e_phentsize);
            let pheader = ElfProgHeader::parse(slice);
            println!("{}", f.fmt_prog_header(&pheader))
        }
    } else {
        assert_eq!(0, f.header.e_phnum); /* extra conformity check */
    }
}

fn print_sections(f: &ElfFile) {
    if f.header.e_shoff != 0 {
        assert_ne!(0, f.header.e_shnum); /* TODO: valid, not supported yet */
        assert!(f.header.e_shnum < elfrust::SHN_LORESERVE); /* extra? conformity check */

        for i in 0..f.header.e_shnum {
            let offset = f.header.e_shoff + u64::from(i) * u64::from(f.header.e_shentsize);
            let sheader = ElfSectionHeader::parse(f.mmap.offlen(offset, f.header.e_shentsize));
            println!("{}", f.fmt_section_header(&sheader))
        }

    } else {
        assert_eq!(0, f.header.e_shnum); /* extra conformity check */
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        panic!("bad number of arguments");
    }

    let f = ElfFile::open(&args[2])?;

    match args[1].as_str() {
        "ident" => println!("{}", f.ident),
        "header" => println!("{}", f.header),

        "segments" => print_segments(&f),

        "sections" => print_sections(&f),

        "all" => {
            println!("{}", f.ident);
            println!("{}", f.header);
            print_segments(&f);
            print_sections(&f)
        }

        _ => panic!("unknown command"),
    }

    Ok(())
}
