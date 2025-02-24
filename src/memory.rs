use std::num::NonZeroUsize;
use std::os::fd::OwnedFd;
use std::ptr::NonNull;
use libc::size_t;
use nix::sys::mman::shm_unlink;
use nix::sys::mman::MapFlags;
use nix::sys::stat::Mode;
use nix::sys::mman::shm_open;
use nix::fcntl::OFlag;
use core::ffi::c_void;
use nix::sys::mman::mmap;
use nix::sys::mman::ProtFlags;
use rand::Rng;

pub fn fill_buffer_random(buf: &mut[u8], stride: u32, height: u32) -> &mut[u8] {
    let mut rng = rand::rng();
    let gcd = num::integer::gcd(stride, height);
    for i in (0..buf.len()).step_by((gcd * 4) as usize) {
	let red_value = rng.random_range(0..255);
	let green_value = rng.random_range(0..255);
	let blue_value = rng.random_range(0..255);
	for ii in (0..gcd).step_by(4) {
	    buf[i + ii as usize] = 255;
	    buf[i + ii as usize + 1] = red_value;
	    buf[i + ii as usize + 2] = green_value;
	    buf[i + ii as usize + 2] = blue_value;
	}
    }
    return buf;
}

#[derive(Debug)]
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

    pub fn destroy(&self) -> Result<(), nix::errno::Errno>{
	unsafe {
	    if let Err(error) = nix::sys::mman::munmap(self.ptr, self.size as size_t) {
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
