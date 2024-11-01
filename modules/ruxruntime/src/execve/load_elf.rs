use alloc::{vec, vec::Vec};
use core::ptr::null_mut;

use crate::execve::custom::*;
use crate::execve::utils::{PROT_EXEC, PROT_READ, PROT_WRITE};
use ruxtask::fs::{get_file_like, Directory, File};

#[derive(Debug)]
pub struct ElfProg {
    pub base: usize,
    pub entry: usize,
    pub interp_path: Vec<u8>,
    pub phent: usize,
    pub phnum: usize,
    pub phdr: usize,
}

impl ElfProg {
    /// read elf from `path`, and copy LOAD segments to a alloacated memory
    ///
    /// and load interp, if needed.
    pub fn new(filepath: &str) -> Self {
        debug!("sys_execve: new elf prog: {filepath}");

        // open file
        let fd = cus_open(filepath, 2, 0);

        // get file size
        let filesize = get_file_like(fd).unwrap().stat().unwrap().st_size as usize;

        // read file
        let mut file = vec![0u8; filesize];
        let _ = cus_read(fd, file.as_mut_ptr(), filesize);
        debug!("sys_execve: read file size 0x{filesize:x}");
        cus_close(fd);

        // parse elf
        let file = elf::ElfBytes::<elf::endian::AnyEndian>::minimal_parse(&file)
            .expect("parse ELF failed");

        // get program's LOAD mem size
        let mut min_addr = 0;
        let mut max_addr = 0;
        let segs = file.segments().unwrap();
        for seg in segs {
            debug!("lhw debug in elf prog vaddr {:X} - {:X}",seg.p_vaddr, seg.p_vaddr + seg.p_memsz);
            if seg.p_type == elf::abi::PT_LOAD {
                min_addr = min_addr.min(seg.p_vaddr);
                max_addr = max_addr.max(seg.p_vaddr + seg.p_memsz);
            }
        }
        let msize = (max_addr - min_addr) as usize;

        // alloc memory for LOAD
        let prot = PROT_WRITE | PROT_READ | PROT_EXEC;
        let flags = MAP_ANONYMOUS | MAP_PRIVATE;
        let base = cus_mmap(null_mut(), msize, prot as _, flags as _, -1, 0);

        // copy LOAD segments
        for seg in segs {
            if seg.p_type == elf::abi::PT_LOAD {
                let data = file.segment_data(&seg).unwrap();
                let dst = (seg.p_vaddr as usize + base) as *mut u8;
                unsafe { dst.copy_from_nonoverlapping(data.as_ptr(), data.len()) };
            }
        }

        // phdr
        let phdr = base + file.ehdr.e_phoff as usize;

        // get entry
        let entry = file.ehdr.e_entry as usize + base;

        // parse interpreter
        let mut interp_path = vec![];
        for seg in file.segments().unwrap() {
            if seg.p_type == elf::abi::PT_INTERP {
                let data = file.segment_data(&seg).unwrap().to_vec();
                interp_path = data;
                break;
            }
        }

        // get address of .text for debugging
        let text_section_addr = base
            + file
                .section_header_by_name(".text")
                .unwrap()
                .unwrap()
                .sh_offset as usize;
        debug!(
            "sys_execve: loaded ELF in 0x{:x}, .text is 0x{:x}",
            base, text_section_addr
        );

        // create retval
        Self {
            base,
            entry,
            interp_path,
            phent: file.ehdr.e_phentsize as usize,
            phnum: file.ehdr.e_phnum as usize,
            phdr,
        }
    }
}
