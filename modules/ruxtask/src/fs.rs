/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! File system related functions.

#![cfg(feature = "fs")]

use crate::current;
use alloc::{format, string::String, sync::Arc, vec::Vec};
use axerrno::{ax_err, AxResult};
use axfs_vfs::VfsNodeRef;
use flatten_objects::FlattenObjects;
use ruxfdtable::FileLike;
use ruxfs::{
    root::{lookup, CurrentWorkingDirectoryOps, RootDirectory},
    MountPoint,
};

use axerrno::{LinuxError, LinuxResult};
use axio::PollState;
use ruxfdtable::RuxStat;
use spin::RwLock;

#[crate_interface::def_interface]
/// The interface for initializing the file system.
pub trait InitFs {
    /// Initializes the file system.
    fn add_stdios_to_fd_table(task_inner: &mut FileSystem);
}

#[cfg(not(feature = "notest"))]
struct InitFsDefaultImpl;

#[cfg(not(feature = "notest"))]
#[crate_interface::impl_interface]
impl InitFs for InitFsDefaultImpl {
    fn add_stdios_to_fd_table(_task_inner: &mut FileSystem) {
        // do nothing
    }
}

/// Initializes the file system.
pub fn get_file_like(fd: i32) -> LinuxResult<Arc<dyn FileLike>> {
    // let _exec = *MUST_EXEC;
    let binding_task = current();
    let mut binding_fs = binding_task.fs.lock();
    if let Some(fs) = binding_fs.as_mut() {
        fs.fd_table
            .get(fd as usize)
            .cloned()
            .ok_or(LinuxError::EBADF)
    } else {
        Err(LinuxError::EBADF)
    }
}

/// Adds a file like object to the file descriptor table and returns the file descriptor.
pub fn add_file_like(f: Arc<dyn FileLike>) -> LinuxResult<i32> {
    let binding_task = current();
    let mut binding_fs = binding_task.fs.lock();
    let fd_table = &mut binding_fs.as_mut().expect("No fd table found").fd_table;
    Ok(fd_table.add(f).ok_or(LinuxError::EMFILE)? as i32)
}

/// Removes a file like object from the file descriptor table.
pub fn close_file_like(fd: i32) -> LinuxResult {
    let binding_task = current();
    let mut binding_fs = binding_task.fs.lock();
    let fd_table = &mut binding_fs.as_mut().unwrap().fd_table;
    let _ = fd_table.remove(fd as usize).ok_or(LinuxError::EBADF)?;
    Ok(())
}

/// A struct representing a file object.
pub struct File {
    /// The inner file object.
    pub inner: RwLock<ruxfs::fops::File>,
}

impl File {
    /// Creates a new file object with the given inner file object.
    pub fn new(inner: ruxfs::fops::File) -> Self {
        Self {
            inner: RwLock::new(inner),
        }
    }

    /// Adds the file object to the file descriptor table and returns the file descriptor.
    pub fn add_to_fd_table(self) -> LinuxResult<i32> {
        add_file_like(Arc::new(self))
    }

    /// Creates a new file object from the given file descriptor.
    pub fn from_fd(fd: i32) -> LinuxResult<Arc<Self>> {
        let f = get_file_like(fd)?;
        f.into_any()
            .downcast::<Self>()
            .map_err(|_| LinuxError::EINVAL)
    }
}

impl FileLike for File {
    fn read(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        Ok(self.inner.write().read(buf)?)
    }

    fn write(&self, buf: &[u8]) -> LinuxResult<usize> {
        Ok(self.inner.write().write(buf)?)
    }

    fn flush(&self) -> LinuxResult {
        Ok(self.inner.write().flush()?)
    }

    fn stat(&self) -> LinuxResult<RuxStat> {
        let metadata = self.inner.read().get_attr()?;
        let ty = metadata.file_type() as u8;
        let perm = metadata.perm().bits() as u32;
        let st_mode = ((ty as u32) << 12) | perm;

        // Inode of files, for musl dynamic linker.
        // WARN: there will be collision for files with the same size.
        // TODO: implement real inode.
        let st_ino = metadata.size() + st_mode as u64;

        let res = RuxStat {
            st_ino,
            st_nlink: 1,
            st_mode,
            st_uid: 1000,
            st_gid: 1000,
            st_size: metadata.size() as _,
            st_blocks: metadata.blocks() as _,
            st_blksize: 512,
            ..Default::default()
        };

        Ok(res)
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        Ok(PollState {
            readable: true,
            writable: true,
            pollhup: false,
        })
    }

    fn set_nonblocking(&self, _nonblocking: bool) -> LinuxResult {
        Ok(())
    }
}

/// A struct representing a directory object.
pub struct Directory {
    /// The inner directory object.
    pub inner: RwLock<ruxfs::fops::Directory>,
}

impl Directory {
    /// Creates a new directory object with the given inner directory object.
    pub fn new(inner: ruxfs::fops::Directory) -> Self {
        Self {
            inner: RwLock::new(inner),
        }
    }

    /// Adds the directory object to the file descriptor table and returns the file descriptor.
    pub fn add_to_fd_table(self) -> LinuxResult<i32> {
        add_file_like(Arc::new(self))
    }

    /// Creates a new directory object from the given file descriptor.
    pub fn from_fd(fd: i32) -> LinuxResult<Arc<Self>> {
        let f = get_file_like(fd)?;
        f.into_any()
            .downcast::<Self>()
            .map_err(|_| LinuxError::EINVAL)
    }
}

impl FileLike for Directory {
    fn read(&self, _buf: &mut [u8]) -> LinuxResult<usize> {
        Err(LinuxError::EACCES)
    }

    fn write(&self, _buf: &[u8]) -> LinuxResult<usize> {
        Err(LinuxError::EACCES)
    }

    fn flush(&self) -> LinuxResult {
        Ok(())
    }

    fn stat(&self) -> LinuxResult<RuxStat> {
        let metadata = self.inner.read().get_attr()?;
        let ty = metadata.file_type() as u8;
        let perm = metadata.perm().bits() as u32;
        let st_mode = ((ty as u32) << 12) | perm;
        Ok(RuxStat {
            st_ino: 1,
            st_nlink: 1,
            st_mode,
            st_uid: 1000,
            st_gid: 1000,
            st_size: metadata.size() as _,
            st_blocks: metadata.blocks() as _,
            st_blksize: 512,
            ..Default::default()
        })
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        Ok(PollState {
            readable: true,
            writable: true,
            pollhup: false,
        })
    }

    fn set_nonblocking(&self, _nonblocking: bool) -> LinuxResult {
        Ok(())
    }
}

/// Maximum number of files per process
pub const RUX_FILE_LIMIT: usize = 1024;

/// A struct representing a file system object.
pub struct FileSystem {
    /// The file descriptor table.
    pub fd_table: FlattenObjects<Arc<dyn FileLike>, RUX_FILE_LIMIT>,
    /// The current working directory.
    pub current_path: String,
    /// The current directory.
    pub current_dir: VfsNodeRef,
    /// The root directory.
    pub root_dir: Arc<RootDirectory>,
}

impl FileSystem {
    /// Closes all file objects in the file descriptor table.
    pub fn close_all_files(&mut self) {
        for fd in 0..self.fd_table.capacity() {
            if self.fd_table.get(fd).is_some() {
                self.fd_table.remove(fd).unwrap();
            }
        }
    }
}

impl Clone for FileSystem {
    fn clone(&self) -> Self {
        let mut new_fd_table = FlattenObjects::new();
        // get all file descriptors from the original file system to copy them to the new one
        // TODO: make this more efficient by only copying the used file descriptors
        for fd in 0..self.fd_table.capacity() {
            if let Some(f) = self.fd_table.get(fd) {
                new_fd_table.add_at(fd, f.clone()).unwrap();
            }
        }

        Self {
            fd_table: new_fd_table,
            current_path: self.current_path.clone(),
            current_dir: self.current_dir.clone(),
            root_dir: self.root_dir.clone(),
        }
    }
}

/// Initializes the file system.
pub fn init_rootfs(mount_points: Vec<MountPoint>) {
    let main_fs = mount_points
        .first()
        .expect("No filesystem found")
        .fs
        .clone();
    let mut root_dir = RootDirectory::new(main_fs);

    for mp in mount_points.iter().skip(1) {
        let path = mp.path;
        let vfsops = mp.fs.clone();
        let message = format!("failed to mount filesystem at {}", path);
        info!("mounting {}", path);
        root_dir.mount(path, vfsops).expect(&message);
    }

    let root_dir_arc = Arc::new(root_dir);

    let mut fs = FileSystem {
        fd_table: FlattenObjects::new(),
        current_path: "/".into(),
        current_dir: root_dir_arc.clone(),
        root_dir: root_dir_arc.clone(),
    };

    // TODO: make a more clear interface for adding stdios to fd table when not in unit tests
    let fs_mutable = &mut fs;
    crate_interface::call_interface!(InitFs::add_stdios_to_fd_table, fs_mutable);

    current().fs.lock().replace(fs);
}

fn parent_node_of(dir: Option<&VfsNodeRef>, path: &str) -> VfsNodeRef {
    if path.starts_with('/') {
        current().fs.lock().as_mut().unwrap().root_dir.clone()
    } else {
        dir.cloned()
            .unwrap_or_else(|| current().fs.lock().as_mut().unwrap().current_dir.clone())
    }
}

/// Returns the absolute path of the given path.
pub fn absolute_path(path: &str) -> AxResult<String> {
    if path.starts_with('/') {
        Ok(axfs_vfs::path::canonicalize(path))
    } else {
        let path = current().fs.lock().as_mut().unwrap().current_path.clone() + path;
        Ok(axfs_vfs::path::canonicalize(&path))
    }
}

/// Returns the current directory.
pub fn current_dir() -> AxResult<String> {
    Ok(current().fs.lock().as_mut().unwrap().current_path.clone())
}

/// Sets the current directory.
pub fn set_current_dir(path: &str) -> AxResult {
    let mut abs_path = absolute_path(path)?;
    if !abs_path.ends_with('/') {
        abs_path += "/";
    }
    if abs_path == "/" {
        current().fs.lock().as_mut().unwrap().current_dir =
            current().fs.lock().as_mut().unwrap().root_dir.clone();
        current().fs.lock().as_mut().unwrap().current_path = "/".into();
        return Ok(());
    }

    let node = lookup(None, &abs_path)?;
    let attr = node.get_attr()?;
    if !attr.is_dir() {
        ax_err!(NotADirectory)
    } else if !attr.perm().owner_executable() {
        ax_err!(PermissionDenied)
    } else {
        current().fs.lock().as_mut().unwrap().current_dir = node;
        current().fs.lock().as_mut().unwrap().current_path = abs_path;
        Ok(())
    }
}

struct CurrentWorkingDirectoryImpl;

#[crate_interface::impl_interface]
impl CurrentWorkingDirectoryOps for CurrentWorkingDirectoryImpl {
    fn init_rootfs(mount_points: Vec<MountPoint>) {
        init_rootfs(mount_points)
    }
    fn parent_node_of(dir: Option<&VfsNodeRef>, path: &str) -> VfsNodeRef {
        parent_node_of(dir, path)
    }
    fn absolute_path(path: &str) -> AxResult<String> {
        absolute_path(path)
    }
    fn current_dir() -> AxResult<String> {
        current_dir()
    }
    fn set_current_dir(path: &str) -> AxResult {
        set_current_dir(path)
    }
    fn root_dir() -> Arc<RootDirectory> {
        current()
            .fs
            .lock()
            .as_mut()
            .expect("No filesystem found")
            .root_dir
            .clone()
    }
}
