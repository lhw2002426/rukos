/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

/// When implement [`VfsNodeOps`] on a directory node, add dummy file operations
/// that just return an error.
///
/// [`VfsNodeOps`]: crate::VfsNodeOps
#[macro_export]
macro_rules! impl_vfs_dir_default {
    () => {
        fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> $crate::VfsResult<usize> {
            $crate::__priv::ax_err!(IsADirectory)
        }

        fn write_at(&self, _offset: u64, _buf: &[u8]) -> $crate::VfsResult<usize> {
            $crate::__priv::ax_err!(IsADirectory)
        }

        fn fsync(&self) -> $crate::VfsResult {
            $crate::__priv::ax_err!(IsADirectory)
        }

        fn truncate(&self, _size: u64) -> $crate::VfsResult {
            $crate::__priv::ax_err!(IsADirectory)
        }

        #[inline]
        fn as_any(&self) -> &dyn core::any::Any {
            self
        }
    };
}

/// When implement [`VfsNodeOps`] on a non-directory node, add dummy directory
/// operations that just return an error.
///
/// [`VfsNodeOps`]: crate::VfsNodeOps
#[macro_export]
macro_rules! impl_vfs_non_dir_default {
    () => {
        fn lookup(
            self: $crate::__priv::Arc<Self>,
            _path: &str,
        ) -> $crate::VfsResult<$crate::VfsNodeRef> {
            $crate::__priv::ax_err!(NotADirectory)
        }

        fn create(&self, _path: &str, _ty: $crate::VfsNodeType) -> $crate::VfsResult {
            $crate::__priv::ax_err!(NotADirectory)
        }

        fn remove(&self, _path: &str) -> $crate::VfsResult {
            $crate::__priv::ax_err!(NotADirectory)
        }

        fn read_dir(
            &self,
            _start_idx: usize,
            _dirents: &mut [$crate::VfsDirEntry],
        ) -> $crate::VfsResult<usize> {
            $crate::__priv::ax_err!(NotADirectory)
        }

        #[inline]
        fn as_any(&self) -> &dyn core::any::Any {
            self
        }
    };
}
