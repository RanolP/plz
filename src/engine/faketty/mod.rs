use std::{os::unix::prelude::RawFd, process};

use nix::{
    fcntl::{fcntl, FcntlArg, FdFlag},
    pty::{self, ForkptyResult, Winsize},
    sys::wait::{self, WaitStatus},
    unistd::{self, Pid},
};

pub fn dup(fd: RawFd) -> nix::Result<RawFd> {
    let new = unistd::dup(fd)?;
    fcntl(new, FcntlArg::F_SETFD(FdFlag::FD_CLOEXEC))?;
    Ok(new)
}

pub unsafe fn forkpty() -> nix::Result<ForkptyResult> {
    let winsize = Winsize {
        ws_row: 24,
        ws_col: 80,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let termios = None;
    pty::forkpty(&winsize, termios)
}

pub fn copyfd(read: RawFd, write: RawFd) {
    const BUF: usize = 4096;
    let mut buf = [0; BUF];
    loop {
        match unistd::read(read, &mut buf) {
            Ok(0) | Err(_) => return,
            Ok(n) => {
                let _ = write_all(write, &buf[..n]);
            }
        }
    }
}

fn write_all(fd: RawFd, mut buf: &[u8]) -> nix::Result<()> {
    while !buf.is_empty() {
        let n = unistd::write(fd, buf)?;
        buf = &buf[n..];
    }
    Ok(())
}

pub fn copyexit(child: Pid) -> ! {
    let flag = None;
    process::exit(match wait::waitpid(child, flag) {
        Ok(WaitStatus::Exited(_pid, code)) => code,
        _ => 0,
    });
}
