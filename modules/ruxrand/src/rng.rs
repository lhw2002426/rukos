use lazy_init::LazyInit;
use log::debug;
use percpu::def_percpu;
use rand::{distributions::Standard, prelude::*};
use ruxdriver::prelude::*;

#[def_percpu]
pub(crate) static PERCPU_RNG: LazyInit<AxRngDevice> = LazyInit::new();

/// Initializes the random number generator device.
pub fn init(rng_dev: AxRngDevice, cpuid: usize) {
    // Initialize the per-CPU RNG for the given CPU ID.
    // This is a placeholder function and should be implemented
    // to initialize the RNG for the specific CPU.
    debug!("Initializing per-CPU RNG for CPU ID: {}", cpuid);
    PERCPU_RNG.with_current(|percpu_ref| {
        percpu_ref.init_by(rng_dev);
    });
}

/// Returns a 32-bit random number using the per-CPU RNG.
// TODO: Implement the actual RNG initialization logic
pub fn percpu_rng() {
    todo!("Implement the per-CPU RNG initialization logic");
}

/// A simple random number generator that uses the per-CPU RNG.
pub struct PercpuRng;

/// Generates a random value of type `T`
pub fn random<T>() -> T
where
    Standard: rand::distributions::Distribution<T>,
{
    let size = core::mem::size_of::<T>();
    let mut buf = [0u8; 64]; // 假设我们最大支持 64 字节
    assert!(size <= 64, "Requested type is too big");

    let dst = &mut buf[..size];
    let _len = PERCPU_RNG.with_current(|r| r.request_entropy(dst)).unwrap();
    let value = unsafe { core::ptr::read_unaligned(dst.as_ptr() as *const T) };
    value
}

/// Requests entropy from the RNG and fills the provided buffer.
pub fn request_entropy(dst: &mut [u8]) -> DevResult<usize> {
    PERCPU_RNG.with_current(|r| r.request_entropy(dst))
}
