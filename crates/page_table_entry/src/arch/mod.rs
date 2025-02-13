/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[doc(cfg(any(target_arch = "riscv32", target_arch = "riscv64")))]
pub mod riscv;

// TODO: `#[cfg(any(target_arch = "aarch64", doc))]` does not work.
#[doc(cfg(target_arch = "aarch64"))]
pub mod aarch64;
