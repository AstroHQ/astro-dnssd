use crate::Result;
use std::time::Duration;

#[cfg(target_os = "windows")]
mod os {
    use super::*;

    // use std::os::windows::io::RawSocket;
    use winapi::um::winsock2::{WSAPoll, POLLRDNORM, SOCKET, SOCKET_ERROR, WSAPOLLFD};
    pub fn socket_is_ready(socket: SOCKET, timeout: Duration) -> Result<bool> {
        let info = WSAPOLLFD {
            fd: socket,
            events: POLLRDNORM,
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
            let ready_to_read = info.revents & POLLRDNORM;
            println!(
                "Some ready, checking flags: {:b} vs {:b}",
                info.revents, POLLRDNORM
            );
            Ok(ready_to_read != 0)
        } else if r == SOCKET_ERROR {
            Err(crate::DNSServiceError::ServiceError(r))
        } else {
            println!("Nothing ready");
            Ok(false)
        }
    }
}
#[cfg(not(target_os = "windows"))]
mod os {
    use super::*;
    pub fn socket_is_ready(socket: i32, timeout: Duration) -> Result<bool> {
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
            return Ok(libc::FD_ISSET(fd, &mut read_set));
        }
        // Ok(false)
    }
}

pub use os::socket_is_ready;
