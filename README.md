# elfrust

A program for inspecting ELF files, written in Rust.

Provides nothing that `readelf` or `objdump` can't do, but allows me to study
the ELF file format and program loading on Linux, as well as to practice Rust.

# Example

The following command prints the ELF header and the segment table from the
`tests/data/hello` ELF executable.

```console
$ elfrust --header --segments tests/data/hello
Elf Class: 64 bits
Data Encoding: 2's complement, least significant byte first
Elf Version: EV_CURRENT
OS ABI: 0
ABI Version: 0
Type: DYN
Machine: 62
Elf Version: 1
Entry: 0x1050
Program Header Offset: 64
Section Header Offset: 13968
Flags: 0
Elf Header Size: 64
Program Header Entry Size: 56
Program Header Number: 14
Section Header Entry Size: 64
Section Header Number: 31
Section Header String Index: 30
 type           | flags   | offset | vaddr  | paddr  | filesz | memsz | align 
----------------+---------+--------+--------+--------+--------+-------+-------
 PHDR           | {R}     |     64 |   0x40 |   0x40 |    784 |   784 |     8 
 INTERP         | {R}     |    916 |  0x394 |  0x394 |     28 |    28 |     1 
 LOAD           | {R}     |      0 |    0x0 |    0x0 |   1576 |  1576 |  4096 
 LOAD           | {X | R} |   4096 | 0x1000 | 0x1000 |    349 |   349 |  4096 
 LOAD           | {R}     |   8192 | 0x2000 | 0x2000 |    268 |   268 |  4096 
 LOAD           | {W | R} |  11728 | 0x3dd0 | 0x3dd0 |    584 |   592 |  4096 
 DYNAMIC        | {W | R} |  11744 | 0x3de0 | 0x3de0 |    480 |   480 |     8 
 NOTE           | {R}     |    848 |  0x350 |  0x350 |     32 |    32 |     8 
 NOTE           | {R}     |    880 |  0x370 |  0x370 |     36 |    36 |     4 
 NOTE           | {R}     |   8428 | 0x20ec | 0x20ec |     32 |    32 |     4 
 OS(1685382483) | {R}     |    848 |  0x350 |  0x350 |     32 |    32 |     8 
 OS(1685382480) | {R}     |   8212 | 0x2014 | 0x2014 |     44 |    44 |     4 
 OS(1685382481) | {W | R} |      0 |    0x0 |    0x0 |      0 |     0 |    16 
 OS(1685382482) | {R}     |  11728 | 0x3dd0 | 0x3dd0 |    560 |   560 |     1 

```

# License

Licensed under either [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE) license.

Copyright 2025-2026 Benjamin Terra-Jorge
