#![feature(range_into_bounds)]

use std::borrow::Cow;
use std::ops::{Bound, IntoBounds, RangeBounds};
use std::{env, fs::File};
use std::error::Error;
use elfrust::{self, ElfFile, ElfProgHeader, ElfSectionHeader};
use tabled::Tabled;
use tabled::grid::records::Records;
use tabled::settings::{object::{Object, FirstRow}, Alignment};
use tabled::{builder::Builder, settings::style::Style, Table};
use tabled::grid::config::Entity;
use pager::Pager;

struct Columns<R: RangeBounds<usize>>(R);

impl<R, Rec> Object<Rec> for Columns<R>
where
    R: IntoBounds<usize> + Clone,
    Rec: Records,
{
    type Iter = Box<dyn Iterator<Item = Entity>>;

    fn cells(&self, rec: &Rec) -> Self::Iter {
        let i : Box<dyn Iterator<Item = usize>> =
            match (0..rec.count_columns()).intersect(self.0.clone()) {
                (Bound::Unbounded, _) | (_, Bound::Unbounded) => panic!("impossible"),
                (Bound::Excluded(_), _) => panic!("impossible?"),
                (Bound::Included(s), Bound::Included(e)) => Box::new((s..=e).into_iter()),
                (Bound::Included(s), Bound::Excluded(e)) => Box::new((s..e).into_iter()),
            };

        Box::new(i.map(|i| Entity::Column(i)))
    }
}

fn fields<'a>(e: &ElfFile, h: &'a ElfSectionHeader) -> Vec<Cow<'a, str>>{
    let mut v = h.fields();
    let name = v.first_mut().unwrap();
    *name = Cow::from(format!("{:?}", e.get_section_name(h.name)));
    v
}

fn print_section_header_table(e: &ElfFile, sh: &[ElfSectionHeader]) -> String {
    let tabled : Vec<Vec<String>> =
        sh.iter().map(|h| fields(e, h).into_iter().map(|f| String::from(f)).collect()).collect();

    let mut builder = Builder::from(tabled);
    builder.insert_record(0, ElfSectionHeader::headers());

    builder.index().build()
        .with(Style::psql())
        .modify(Columns(4..), Alignment::right())
        .modify(FirstRow, Alignment::left())
        .to_string()
}

fn print_program_header_table(ph: &[ElfProgHeader]) -> String {
    Table::new(ph)
        .with(Style::psql())
        .modify(Columns(2..), Alignment::right())
        .modify(FirstRow, Alignment::left())
        .to_string()
}

fn print_segments (f: &ElfFile) {
    let ph = f.program_headers();
    if ! ph.is_empty() {
        println!("{}", print_program_header_table(ph))
    }
}

fn print_sections(f: &ElfFile) {
    let sh = f.section_headers();
    if ! sh.is_empty() {
        println!("{}", print_section_header_table(&f, sh))
    }
}

fn print_strings(f: &ElfFile) {
    for st in f.string_table_sections() {
        println!("{:?}:", f.get_section_name(st.name));

        for (i, s) in &st {
            println!("{:6}: {:?}", i, s);
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    Pager::with_pager("less -SF").setup();

    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        panic!("bad number of arguments");
    }

    let f = File::open(&args[2])?;

    match args[1].as_str() {
        "ident" => println!("{}", ElfFile::read_elf_header(&f)?.e_ident),
        "header" => println!("{}", ElfFile::read_elf_header(&f)?),
        "interp" => println!("{:?}", ElfFile::from_file(f)?.interp()?.unwrap()),

        "segments" => {
            let f = ElfFile::from_file(f)?;
            print_segments(&f)
        },

        "sections" => {
            let f = ElfFile::from_file(f)?;
            print_sections(&f)
        },

        "strings" => print_strings(&ElfFile::from_file(f)?),

        "all" => {
            let f = ElfFile::from_file(f)?;
            println!("{}", f.header().e_ident);
            println!("{}", f.header());
            println!("{:?}", f.interp()?.unwrap());
            print_segments(&f);
            print_sections(&f);
            print_strings(&f)
        }

        _ => panic!("unknown command"),
    }

    Ok(())
}
