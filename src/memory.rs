use std::num::NonZeroUsize;
use std::os::fd::OwnedFd;
use std::ptr::NonNull;
use libc::off_t;
use nix::sys::mman::MapFlags;
use nix::sys::stat::Mode;
use nix::sys::mman::shm_open;
use nix::fcntl::OFlag;
use core::ffi::c_void;
use nix::sys::mman::mmap;
use nix::sys::mman::ProtFlags;

#[derive(Debug)]
pub struct MemoryMapping {
    name: String,
    pub fd: OwnedFd,
    pub ptr: NonNull<c_void>,
    pub size: NonZeroUsize
}

impl MemoryMapping {
    pub fn new(name: String, size: NonZeroUsize) -> Option<Self> {
	let fd = match shm_open(name.as_str(), OFlag::O_RDWR | OFlag::O_CREAT | OFlag::O_TRUNC, Mode::S_IRWXU) {
	    Ok(result) => result,
	    Err(error) => {
		println!("Error with shm_open : {}", error);
		return None;
	    }
	};

	let size_as_off_t: off_t = match off_t::try_from(size.get()) {
	    Ok(size_as_off_t) => size_as_off_t,
	    Err(error) => panic!("failed to convert usize to off_t: {}", error),
};

	if let Result::Err(errno) = nix::unistd::ftruncate(&fd, size_as_off_t) {
	    println!("Failed to ftruncate ! : {}", errno);
	    return None;
	}

	unsafe {
	    let ptr: NonNull<c_void> = match mmap(
		None,
		size,
		ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
		MapFlags::MAP_SHARED,
		&fd, 0
	    ) {
		Ok(ptr) => ptr,
		Err(errno) => {
		    println!("Failed to mmap ! : {}", errno);
		    return None;
		}
	    };
	    Some(MemoryMapping {
		name,
		fd,
		ptr,
		size
	    })
	}
    }

    pub fn destroy(&self) -> Result<(), nix::errno::Errno>{
	unsafe {
	    if let Err(error) = nix::sys::mman::munmap(self.ptr, self.size.get()) {
		println!("munmap failed {}", error);
		return Err(error);
	    }
	}
	if let Err(error) = nix::sys::mman::shm_unlink(self.name.as_str()) {
	    println!("shm_unlink failed {}", error);
	    return Err(error);
	}
	return Result::Ok(());
    }
}
