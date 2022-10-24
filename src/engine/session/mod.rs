use std::{
    ffi::{CStr, CString},
    fs::File,
    io::{self, Write},
    os::unix::{
        prelude::{IntoRawFd, OsStrExt},
        process::ExitStatusExt,
    },
    path::{Path, PathBuf},
    process::ExitStatus,
    sync::mpsc::{self, Receiver},
    thread::{spawn},
};

pub use error::SessionError;
pub use init::SessionInit;
use nix::{
    fcntl::{open, OFlag},
    libc::{close, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO},
    pty::{grantpt, posix_openpt, ptsname, unlockpt},
    sys::{
        stat::Mode,
        termios::{tcgetattr, tcsetattr, LocalFlags, SetArg, InputFlags, OutputFlags},
        wait::waitpid,
    },
    unistd::{dup2, execvp, fork, fsync, read, write, ForkResult},
};

use self::event::SessionEvent;

mod error;
mod event;
mod init;

pub struct Session {
    pub shell: PathBuf,
}

impl Session {
    pub fn run(self) -> Result<ExitStatus, SessionError> {
        let (tx, rx) = mpsc::channel::<SessionEvent>();

        let stdio_loop = spawn(move || Session::stdio_loop(rx));

        let master_fd = posix_openpt(OFlag::O_RDWR)?;
        grantpt(&master_fd)?;
        unlockpt(&master_fd)?;

        let slave_fd = open(
            Path::new(&unsafe { ptsname(&master_fd) }?),
            OFlag::O_RDWR,
            Mode::empty(),
        )?;

        let mut termios = tcgetattr(STDIN_FILENO)?;

        termios.input_flags.remove(InputFlags::BRKINT);
        termios.input_flags.remove(InputFlags::ICRNL);
        termios.input_flags.remove(InputFlags::INPCK);
        termios.input_flags.remove(InputFlags::ISTRIP);
        termios.input_flags.remove(InputFlags::IXON);
        
        termios.output_flags.remove(OutputFlags::OPOST);

        termios.local_flags.remove(LocalFlags::ICANON);
        termios.local_flags.remove(LocalFlags::ECHO);
        termios.local_flags.remove(LocalFlags::IEXTEN);
        // termios.local_flags.remove(LocalFlags::ISIG);

        tcsetattr(STDIN_FILENO, SetArg::TCSANOW, &termios)?;

        let fork_result = unsafe { fork() }?;

        match fork_result {
            ForkResult::Parent { child } => {
                unsafe { close(slave_fd) };
                let raw_fd = master_fd.into_raw_fd();

                let tx_stdin = tx.clone();
                spawn(move || -> Result<_, SessionError> {
                    let mut buf = [0; 1024];
                    while let Ok(len) = read(STDIN_FILENO, &mut buf).map_err(io::Error::from) {
                        if len <= 0 {
                            break;
                        }
                        write(raw_fd, &buf[..len]).map_err(io::Error::from)?;
                        if buf[..len].iter().any(|b| *b == b'\n') {
                            tx_stdin.send(SessionEvent::Enter)?;
                        }
                    }
                    Ok(())
                });

                let tx_stdout = tx.clone();
                spawn(move || -> Result<_, SessionError> {
                    let mut buf = [0; 1024];
                    while let Ok(len) = read(raw_fd, &mut buf).map_err(io::Error::from) {
                        if len <= 0 {
                            break;
                        }
                        write(STDOUT_FILENO, &buf[..len]).map_err(io::Error::from)?;
                        tx_stdout.send(SessionEvent::Stdout(buf[..len].to_vec()))?;
                    }
                    Ok(())
                });

                waitpid(child, None)?;
                tx.send(SessionEvent::Die)?;
            }
            ForkResult::Child => {
                drop(master_fd);
                dup2(slave_fd, STDIN_FILENO)?;
                dup2(slave_fd, STDOUT_FILENO)?;
                dup2(slave_fd, STDERR_FILENO)?;
                let shell = CString::new(self.shell.as_os_str().as_bytes())?;
                let shell = shell.as_c_str();
                let args: &[&CStr] = &[shell];

                execvp(shell, args)?;
            }
        }

        stdio_loop.join().unwrap()?;

        Ok(ExitStatus::from_raw(0))
    }

    fn stdio_loop(rx: Receiver<SessionEvent>) -> Result<(), SessionError> {
        let mut buffer: Vec<u8> = Vec::new();

        let mut file = File::create("debug.log")?;

        while let Ok(e) = rx.recv() {
            match e {
                SessionEvent::Stdout(data) => {
                    buffer.extend(&data);
                }
                SessionEvent::Stderr(data) => {
                    buffer.extend(&data);
                    // stderr.write(&data)?;
                    // stderr.flush()?;
                }
                SessionEvent::Enter => {
                    file.write(b":::debug (buffer)\n")?;
                    file.write(&buffer)?;
                    buffer.clear();
                    file.write(b"\n:::\n\n")?;
                }
                SessionEvent::Die => break,
            }
        }
        Ok(())
    }
}
