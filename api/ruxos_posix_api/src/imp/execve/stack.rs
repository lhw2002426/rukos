use alloc::{vec, vec::Vec};

const STACK_SIZE: usize = ruxconfig::TASK_STACK_SIZE;

#[derive(Debug)]
pub struct Stack {
    /// stack
    data: Vec<u8>,
    /// index of top byte of stack
    top: usize,
}

#[link_section = ".bss.stack"]
static mut STATIC_STACK:[u8; 0x20000] = [0; 0x20000];

impl Stack {
    /// alloc a stack
    pub fn new() -> Self {
<<<<<<< HEAD
        Self {
            data: vec![0u8; STACK_SIZE],
            top: STACK_SIZE,
        }
=======
        //let size = 0xa00000; // 10M
        //let p = sys_mmap(null_mut(), size, 7, 0x21, -1, 0);
        // unsafe{
        //     let mut vec = core::slice::from_raw_parts_mut(p as *mut u8, size);
        //     vec.fill(0);
        // }

        unsafe{
            let p = STATIC_STACK;
            let size = 0x20000;
            let start = p.as_ptr() as usize;
            let sp = start + size / 2;
            let end = sp;

            error!("sp_addr : {:x?}-{:x?}", start, end);

            Self { sp, start, end }
        }
        
>>>>>>> ysp/python-dyn
    }

    /// addr of top of stack
    pub fn sp(&self) -> usize {
        self.data.as_ptr() as usize + self.top
    }

    /// push data to stack and return the addr of sp
    pub fn push<T>(&mut self, data: &[T], align: usize) -> usize {
        // move sp to right place
        self.top -= core::mem::size_of_val(data);
        self.top = memory_addr::align_down(self.top, align);

        assert!(self.top <= self.data.len(), "sys_execve: stack overflow.");

        // write data into stack
        let sp = self.sp() as *mut T;
        unsafe {
            sp.copy_from_nonoverlapping(data.as_ptr(), data.len());
        }

        sp as usize
    }
}
