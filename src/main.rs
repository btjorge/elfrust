use std::env;
use std::error::Error;
use elfrust::{self, ElfFile, ElfProgHeader, ElfProgHeaderRaw, ElfSectionHeader, OffLen};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("bad number of arguments");
    }

    let f = ElfFile::open(&args[1])?;

    println!("{}", f.ident);

    println!("{:?}", f.header);

    if f.header.e_phoff != 0 {
        assert_ne!(0, f.header.e_phnum);

        /* good if the right align-size equation holds (see above) */
        for i in 0..f.header.e_phnum {
            let offset = f.header.e_phoff + u64::from(i) * u64::from(f.header.e_phentsize);
            let slice = f.mmap.offlen(offset, f.header.e_phentsize);
            let pheader = ElfProgHeader::of_raw(ElfProgHeaderRaw::parse(slice)).ok_or("error")?;
            println!("{}", f.fmt_prog_header(&pheader))
        }
    } else {
        assert_eq!(0, f.header.e_phnum); /* extra conformity check */
    }

    if f.header.e_shoff != 0 {
        assert_ne!(0, f.header.e_shnum); /* TODO: valid, not supported yet */
        assert!(f.header.e_shnum < elfrust::SHN_LORESERVE); /* extra? conformity check */

        for i in 0..f.header.e_shnum {
            let offset = f.header.e_shoff + u64::from(i) * u64::from(f.header.e_shentsize);
            let sheader = ElfSectionHeader::parse(f.mmap.offlen(offset, f.header.e_shentsize));
            println!("{}: {:?} {:?}", i, f.get_section_name(sheader.sh_name), sheader)
        }

    } else {
        assert_eq!(0, f.header.e_shnum); /* extra conformity check */
    }

    Ok(())
}
