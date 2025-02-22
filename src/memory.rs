use std::num::NonZeroUsize;
use std::os::fd::OwnedFd;
use std::ptr::NonNull;
use nix::sys::mman::MapFlags;
use nix::sys::stat::Mode;
use nix::sys::mman::shm_open;
use nix::fcntl::OFlag;
use core::ffi::c_void;
use nix::sys::mman::mmap;
use nix::sys::mman::ProtFlags;


pub struct MemoryMapping {
    name: String,
    pub fd: OwnedFd,
    pub ptr: NonNull<c_void>,
    pub size: usize
}

impl MemoryMapping {
    pub fn new(name: String, size: usize) -> Option<Self> {
	let fd = match shm_open(name.as_str(), OFlag::O_RDWR | OFlag::O_CREAT, Mode::S_IRWXU) {
	    Ok(result) => result,
	    Err(error) => {
		println!("Error with shm_open : {}", error);
		return None;
	    }
	};

	if let Result::Err(errno) = nix::unistd::ftruncate(&fd, size as i64) {
	    println!("Failed to ftruncate ! : {}", errno);
	    return None;
	}

	let nonzerosize= match NonZeroUsize::new(size) {
	    Some(nonzerosize) => nonzerosize,
	    None => {
		println!("size is 0 !");
		return None;
	    }
	};

	println!("nonzerosize = {nonzerosize}");

	unsafe {
	    let ptr: NonNull<c_void> = match mmap(
		None,
		nonzerosize,
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
}
