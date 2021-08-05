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
    pub fn socket_is_ready(
        socket: i32,
        timeout: std::time::Duration,
    ) -> Result<bool, std::io::Error> {
        unsafe {
            let fd = socket;
            let mut timeout = libc::timeval {
                tv_sec: timeout.as_secs() as _,
                tv_usec: timeout.as_micros() as _,
            };
            let mut read_set = std::mem::zeroed();
            libc::FD_ZERO(&mut read_set);
            libc::FD_SET(fd, &mut read_set);
            libc::select(
                fd + 1,
                &mut read_set,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut timeout,
            );
            Ok(libc::FD_ISSET(fd, &mut read_set))
        }
    }
}
#[cfg(any(not(target_os = "windows"), feature = "win-bonjour"))]
pub use os::socket_is_ready;
