#![feature(range_into_bounds)]

use std::borrow::Cow;
use std::ops::{Bound, IntoBounds, RangeBounds};
use std::path::PathBuf;
use std::fs::File;
use std::error::Error;
use elfrust::{self, ElfFile, ElfProgHeader, ElfSectionHeader, SymTableSection};
use clap::{Parser, Args};
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

fn section_header_table_fields<'a>(e: &ElfFile, h: &'a ElfSectionHeader) -> Vec<Cow<'a, str>>{
    let mut v = h.fields();
    let name = v.first_mut().unwrap();
    *name = Cow::from(format!("{:?}", e.get_section_name(h.name)));
    v
}

fn print_section_header_table(e: &ElfFile, sh: &[ElfSectionHeader]) -> String {
    let tabled : Vec<Vec<String>> =
        sh.iter().map(|h| section_header_table_fields(e, h).into_iter().map(|f| String::from(f)).collect()).collect();

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

fn print_symbol_table(t: &SymTableSection) -> String {
    Table::new(t)
        .with(Style::psql())
        .modify(Columns(4..), Alignment::right())
        .modify(FirstRow, Alignment::left())
        .to_string()
}

fn print_symbols(e: &ElfFile) {
    for (h, t) in e.symtab_sections().chain(e.dynsym_sections()) {
        println!("{:?}:", e.get_section_name(h.name));

        println!("{}", print_symbol_table(&t));
    }
}

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(flatten)]
    what: What,
    file: PathBuf,
}

#[derive(Args)]
#[group(required = true, multiple = true)]
struct What {
    /// Print ELF identification
    #[arg(long)]
    ident: bool,

    /// Print ELF header
    #[arg(long)]
    header: bool,

    /// Print interpreter
    #[arg(long)]
    interp: bool,

    /// Print the program header as a table
    #[arg(long)]
    segments: bool,

    /// Print the section header table
    #[arg(long)]
    sections: bool,

    /// Print all the string tables
    #[arg(long)]
    strings: bool,

    /// Print the symbols tables
    #[arg(long)]
    symbols: bool,

    /// Print everything
    #[arg(long)]
    all: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    Pager::with_pager("less -SF").setup();

    let cli = Cli::parse();

    let f = ElfFile::from_file(File::open(cli.file)?)?;

    if cli.what.all {
        println!("{}", f.header().e_ident);
        println!("{}", f.header());
        println!("{:?}", f.interp()?.unwrap());
        print_segments(&f);
        print_sections(&f);
        print_strings(&f);
        print_symbols(&f);
        return Ok(())
    }

    if cli.what.ident {
        println!("{}", f.header().e_ident)
    }

    if cli.what.header {
        println!("{}", f.header());
    }

    if cli.what.interp {
        println!("{:?}", f.interp()?.unwrap())
    }

    if cli.what.segments {
        print_segments(&f)
    }

    if cli.what.sections {
        print_sections(&f)
    }

    if cli.what.strings {
        print_strings(&f)
    }

    if cli.what.symbols {
        print_symbols(&f)
    }

    Ok(())
}
