pub const MAP_SHARED: u32 = 1;
pub const MAP_PRIVATE: u32 = 2;
pub const MAP_SHARED_VALIDATE: u32 = 3;
pub const MAP_TYPE: u32 = 15;
pub const MAP_FIXED: u32 = 16;
pub const MAP_FILE: u32 = 0;
pub const MAP_ANONYMOUS: u32 = 32;
pub const MAP_ANON: u32 = 32;
pub const MAP_HUGE_SHIFT: u32 = 26;
pub const MAP_HUGE_MASK: u32 = 63;
use axerrno::{LinuxError, LinuxResult};

use ruxfs::{
    api::set_current_dir,
    fops::{DirEntry, OpenOptions},
};
use ruxtask::fs::{close_file_like, get_file_like, Directory, File};
use ruxtask::vma::Vma;
use ruxtask::{current, vma::FileInfo};

use memory_addr::PAGE_SIZE_4K;
use ruxhal::mem::VirtAddr;
use ruxmm::paging::pte_update_page;

use crate::execve::utils::*;

pub fn cus_open(filename: &str, flags: i32, mode: i32) -> i32 {
    debug!("sys_open <= {:?} {:#o} {:#o}", filename, flags, mode);
    let mut options = OpenOptions::new();
    options.read(true);
    options.write(true);
    let file = ruxfs::fops::File::open(filename, &options).unwrap();
    File::new(file).add_to_fd_table().unwrap()
}

pub fn cus_read(fd: i32, buf: *mut u8, count: usize) -> LinuxResult<usize> {
    debug!("sys_read <= {} {:#x} {}", fd, buf as usize, count);
    if buf.is_null() {
        panic!("read null in start exec");
    }
    let dst = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, count) };
    Ok(get_file_like(fd)?.read(dst)?)
}

pub fn cus_close(fd: i32) -> i32 {
    close_file_like(fd).map(|_| 0).unwrap()
}


pub fn cus_mmap(
    start: *mut usize,
    len: usize,
    prot: u32,
    flags: u32,
    fd: i32,
    off: usize,
) -> usize {
    debug!(
        "sys_mmap <= start: {:p}, len: 0x{:x}, prot:0x{:x?}, flags:0x{:x?}, fd: {}",
        start, len, prot, flags, fd
    );
    // transform C-type into rust-type
    let start = start as usize;
    let len = VirtAddr::from(len).align_up_4k().as_usize();
    if !VirtAddr::from(start).is_aligned(PAGE_SIZE_4K) || len == 0 {
        error!(
            "mmap failed because start:0x{:x} is not aligned or len:0x{:x} == 0",
            start, len
        );
        panic!("mmap failed in start exec");
    }
    let prot = prot as u32;
    let flags = flags as u32;
    let fid = fd;
    let offset = off as usize;

    // check if `MAP_SHARED` or `MAP_PRIVATE` within flags.
    if (flags & MAP_PRIVATE == 0) && (flags & MAP_SHARED == 0) {
        error!("mmap failed because none of `MAP_PRIVATE` and `MAP_SHARED` exist");
        panic!("mmap failed in start exec");
    }

    // check if `MAP_ANOYMOUS` within flags.
    let fid = if flags & MAP_ANONYMOUS != 0 {
        -1
    } else if fid < 0 {
        error!("fd in mmap without `MAP_ANONYMOUS` must larger than 0");
        panic!("mmap failed in start exec");
    } else {
        fid
    };

    let mut new = Vma::new(fid, offset, prot, flags);
    let binding_task = current();
    let mut vma_map = binding_task.mm.vma_map.lock();
    let addr_condition = if start == 0 { None } else { Some(start) };

    let try_addr = if flags & MAP_FIXED != 0 {
        snatch_fixed_region(&mut vma_map, start, len)
    } else {
        find_free_region(&vma_map, addr_condition, len)
    };

    match try_addr {
        Some(vaddr) => {
            new.start_addr = vaddr;
            new.end_addr = vaddr + len;
            vma_map.insert(vaddr, new);
            vaddr
        }
        _ => panic!("mmap failed in start exec: ENOMEM"),
    }
}

use core::sync::atomic::{AtomicU64, Ordering::SeqCst};
static SEED: AtomicU64 = AtomicU64::new(0xae_f3);

/// Returns a 32-bit unsigned pseudo random interger using LCG.
pub fn cus_rand_lcg32() -> u32 {
    let new_seed = SEED
        .load(SeqCst)
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1);
    SEED.store(new_seed, SeqCst);
    (new_seed >> 33) as u32
}