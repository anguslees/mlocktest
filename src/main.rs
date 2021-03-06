use clap::{App, Arg};
use core::ffi::c_void;
use nix::sys::mman::{mlock, mmap, MapFlags, ProtFlags};
use nix::unistd::pause;
use std::fs::OpenOptions;
use std::io::{self, IoSlice, Write};
use std::iter;
use std::os::unix::io::AsRawFd;
use std::ptr;
use std::slice;

fn writen(mut f: impl Write, mut nbytes: usize) -> io::Result<()> {
    const BUFSIZE: usize = 1024 * 1024;
    let data: [u8; BUFSIZE] = [42; BUFSIZE];

    while nbytes >= BUFSIZE {
        let slices: Vec<_> = iter::repeat(IoSlice::new(&data))
            .take(nbytes / BUFSIZE)
            .collect();
        let n = f.write_vectored(&slices)?;
        nbytes -= n;
    }
    f.write_all(&data[0..nbytes])?;
    f.flush()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("mlock test scenario")
        .arg(
            Arg::with_name("size")
                .short("n")
                .long("size")
                .takes_value(true)
                .required(true)
                .help("Size to allocate (MiB)"),
        )
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .takes_value(true)
                .required(true)
                .help("File to generate/mmap"),
        )
        .arg(
            Arg::with_name("lock")
                .long("lock")
                .help("memlock mmaped region"),
        )
        .arg(
            Arg::with_name("pause")
                .long("pause")
                .help("Pause indefinitely rather than exit"),
        )
        .get_matches();

    let do_pause = matches.is_present("pause");
    let do_memlock = matches.is_present("lock");

    let path = matches.value_of("file").expect("--file is required");
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    let size_mb: usize = matches
        .value_of("size")
        .expect("--size is required")
        .parse()?;
    let size = size_mb * 1024 * 1024;
    writen(&file, size)?;

    let mmap_buf = unsafe {
        let ptr = mmap(
            ptr::null::<c_void>() as *mut c_void,
            size,
            ProtFlags::PROT_READ,
            MapFlags::MAP_PRIVATE,
            file.as_raw_fd(),
            0,
        )?;

        // NB: no munmap - leaks mmapped buf
        slice::from_raw_parts::<'static, u8>(ptr as *const u8, size)
    };

    if do_memlock {
        unsafe {
            mlock(mmap_buf.as_ptr() as *const c_void, mmap_buf.len())?;
        }
    }

    if do_pause {
        pause()
    }

    Ok(())
}
