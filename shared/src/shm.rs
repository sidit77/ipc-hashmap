use std::ffi::c_void;
use std::mem::size_of;
use std::os::fd::OwnedFd;
use std::path::{Path, PathBuf};
use std::ptr::null_mut;
use nix::fcntl::OFlag;
use nix::sys::mman::{MapFlags, mmap, munmap, ProtFlags, shm_open, shm_unlink};
use nix::Result;
use nix::sys::stat::Mode;
use nix::unistd::ftruncate;

pub struct SharedMemory<T> {
    owner: Option<PathBuf>,
    _fd: OwnedFd,
    data: *mut T
}

///SAFETY: Must not depend on  `Drop` and all data must live inside an `UnsafeCell`
pub unsafe trait SharedMemorySafe: Sized {}

impl<T: SharedMemorySafe> SharedMemory<T> {

    pub fn create<P: AsRef<Path>>(name: P, read_only: bool, data: T) -> Result<Self> {
        Self::create_or_open(name, read_only, Some(data))
    }

    pub fn open<P: AsRef<Path>>(name: P, read_only: bool) -> Result<Self> {
        Self::create_or_open(name, read_only, None)
    }

    fn create_or_open<P: AsRef<Path>>(name: P, read_only: bool, data: Option<T>) -> Result<Self> {
        let open = data.is_some();
        let o_flag = read_only
            .then_some(OFlag::O_RDONLY)
            .unwrap_or(OFlag::O_RDWR)
            .union(open
                .then_some(OFlag::O_CREAT | OFlag::O_EXCL)
                .unwrap_or(OFlag::empty()));
        let fd = shm_open(name.as_ref(), o_flag, Mode::S_IRUSR | Mode::S_IWUSR)?;
        let size = size_of::<T>();
        if open {
            ftruncate(&fd, size.try_into().unwrap())?;
        }
        let p_flag = read_only
            .then_some(ProtFlags::PROT_READ)
            .unwrap_or(ProtFlags::PROT_READ | ProtFlags::PROT_WRITE);
        let data_ptr = unsafe { mmap(None, size.try_into().unwrap(), p_flag, MapFlags::MAP_SHARED, Some(&fd), 0)? as *mut T };
        if open {
            unsafe { data_ptr.write(data.unwrap()) }
        }

        Ok(Self {
            owner: open.then(|| name.as_ref().to_path_buf()),
            _fd: fd,
            data: data_ptr
        })
    }

}

impl<T> AsRef<T> for SharedMemory<T> {
    fn as_ref(&self) -> &T {
        unsafe { &*self.data }
    }
}

impl<T> Drop for SharedMemory<T> {
    fn drop(&mut self) {
        unsafe {
            munmap(self.data as *mut c_void, size_of::<T>())
                .unwrap_or_else(|err| println!("Failed to unmap shared memory: {}", err));
            self.data = null_mut();
        }
        if let Some(path) = self.owner.take() {
            shm_unlink(&path)
                .unwrap_or_else(|err| println!("Failed to unlink shared memory: {}", err));
        }
    }
}