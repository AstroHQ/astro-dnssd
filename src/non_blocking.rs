#[cfg(all(target_os = "windows", feature = "win-bonjour"))]
mod os {
    use super::*;
    use winapi::um::winsock2::{WSAPoll, POLLIN, SOCKET, SOCKET_ERROR, WSAPOLLFD};

    pub fn socket_is_ready(
        socket: SOCKET,
        timeout: std::time::Duration,
    ) -> Result<bool, std::io::Error> {
        let info = WSAPOLLFD {
            fd: socket,
            events: POLLIN,
            revents: 0,
        };
        let mut sockets = [info];
        let r = unsafe {
            WSAPoll(
                sockets.as_mut_ptr(),
                sockets.len() as u32,
                timeout.as_millis() as i32,
            )
        };
        if r != SOCKET_ERROR && r > 0 {
            // let ready_to_read = info.revents & POLLIN;
            trace!(
                "Some ready, checking flags: {:b} vs {:b}",
                info.revents,
                POLLIN
            );
            // Ok(ready_to_read != 0)
            // TODO: figure out why no flags are set, or maybe switch to IOCP
            Ok(true)
        } else if r == SOCKET_ERROR {
            Err(std::io::Error::from_raw_os_error(r))
        } else {
            trace!("Nothing ready");
            Ok(false)
        }
    }
}
#[cfg(not(target_os = "windows"))]
mod os {
    use std::time::{Duration, Instant};

    /// Waits up to `timeout` for `socket` to have data. Uses poll() rather than select(), which can't take an fd of
    /// 1024 or more, and retries a wait a signal cut short with the time that's left rather than reporting ready
    pub fn socket_is_ready(socket: i32, timeout: Duration) -> Result<bool, std::io::Error> {
        // None: too far off to add to now, wait without a deadline
        let deadline = Instant::now().checked_add(timeout);
        loop {
            let remaining = match deadline {
                Some(deadline) => deadline.saturating_duration_since(Instant::now()),
                None => timeout,
            };
            // round up so a sub-millisecond wait doesn't become a busy loop of 0 ms polls
            let mut millis = remaining.as_millis();
            if remaining.subsec_nanos() % 1_000_000 != 0 {
                millis += 1;
            }
            let millis = millis.min(libc::c_int::MAX as u128) as libc::c_int;
            let mut fds = [libc::pollfd {
                fd: socket,
                events: libc::POLLIN,
                revents: 0,
            }];
            let r = unsafe { libc::poll(fds.as_mut_ptr(), 1, millis) };
            if r > 0 {
                // POLLHUP/POLLERR count as ready too: DNSServiceProcessResult then returns the error at once
                return Ok(fds[0].revents != 0);
            }
            if r == 0 {
                return Ok(false);
            }
            let err = std::io::Error::last_os_error();
            if err.kind() != std::io::ErrorKind::Interrupted {
                return Err(err);
            }
            if matches!(deadline, Some(deadline) if Instant::now() >= deadline) {
                return Ok(false);
            }
        }
    }
}
#[cfg(any(not(target_os = "windows"), feature = "win-bonjour"))]
pub use os::socket_is_ready;

/// Audit repros (Nye)
#[cfg(all(test, not(target_os = "windows")))]
mod audit_repro {
    use super::socket_is_ready;
    use std::time::Duration;

    extern "C" fn noop(_: libc::c_int) {}

    fn idle_socket() -> (i32, i32) {
        let mut fds = [0i32; 2];
        assert_eq!(
            unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()) },
            0
        );
        (fds[0], fds[1])
    }

    /// A-4: select()'s return value is ignored; when a signal interrupts it the fd set is untouched, so we report
    /// "ready" and the caller walks into a blocking DNSServiceProcessResult.
    #[test]
    fn interrupted_select_is_not_ready() {
        let (fd, _peer) = idle_socket();
        unsafe {
            let mut sa: libc::sigaction = std::mem::zeroed();
            sa.sa_sigaction = noop as extern "C" fn(libc::c_int) as usize;
            libc::sigaction(libc::SIGUSR1, &sa, std::ptr::null_mut());
            let me = libc::pthread_self() as usize;
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(200));
                libc::pthread_kill(me as libc::pthread_t, libc::SIGUSR1);
            });
        }
        let ready = socket_is_ready(fd, Duration::from_secs(2)).unwrap_or(false);
        assert!(
            !ready,
            "nothing was written to the socket, yet it's reported ready"
        );
    }

    /// A-5: fds at or above FD_SETSIZE (1024) can't go in an fd_set.
    #[test]
    fn high_fd_does_not_panic() {
        let (fd, _peer) = idle_socket();
        unsafe {
            let lim = libc::rlimit {
                rlim_cur: 4096,
                rlim_max: 4096,
            };
            let mut cur: libc::rlimit = std::mem::zeroed();
            libc::getrlimit(libc::RLIMIT_NOFILE, &mut cur);
            if cur.rlim_cur < 2048 {
                assert_eq!(libc::setrlimit(libc::RLIMIT_NOFILE, &lim), 0);
            }
            let high = libc::fcntl(fd, libc::F_DUPFD, 1500);
            assert!(high >= 1500);
            let r = std::panic::catch_unwind(|| socket_is_ready(high, Duration::from_millis(50)));
            assert!(
                matches!(r, Ok(Ok(false))),
                "socket_is_ready broke on fd {}: {:?}",
                high,
                r.map(|r| r.ok())
            );
        }
    }
}
