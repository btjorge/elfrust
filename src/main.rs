use std::env;
use std::error::Error;
use elfrust::{self, ElfFile};

fn print_segments (f: &ElfFile) {
    let ph = &f.program_headers;
    if ! ph.is_empty() {
        for h in ph {
            println!("{}", f.fmt_prog_header(&h));
        }
    }
}

fn print_sections(f: &ElfFile) {
    let sh = &f.section_headers;
    if ! sh.is_empty () {
        for h in sh {
            println!("{}", f.fmt_section_header(&h));
        }
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
