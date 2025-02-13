/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const NUM_TASKS: usize = 5;

static FINISHED_TASKS: AtomicUsize = AtomicUsize::new(0);

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    println!("Hello, main task!");
    let now = Instant::now();
    thread::sleep(Duration::from_secs(1));
    let elapsed = now.elapsed();
    println!("main task sleep for {:?}", elapsed);

    // backgroud ticks, 0.5s x 30 = 15s
    thread::spawn(|| {
        for i in 0..30 {
            println!("  tick {}", i);
            thread::sleep(Duration::from_millis(500));
        }
    });

    // task n: sleep 3 x n (sec)
    for i in 0..NUM_TASKS {
        thread::spawn(move || {
            let sec = i + 1;
            for j in 0..3 {
                println!("task {} sleep {} seconds ({}) ...", i, sec, j);
                let now = Instant::now();
                thread::sleep(Duration::from_secs(sec as _));
                let elapsed = now.elapsed();
                println!("task {} actual sleep {:?} seconds ({}).", i, elapsed, j);
            }
            FINISHED_TASKS.fetch_add(1, Ordering::Relaxed);
        });
    }

    while FINISHED_TASKS.load(Ordering::Relaxed) < NUM_TASKS {
        thread::sleep(Duration::from_millis(10));
    }
    println!("Sleep tests run OK!");
}
