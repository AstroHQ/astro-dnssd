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
    pub fn socket_is_ready(socket: u32, timeout: Duration) -> Result<bool> {
        // unsafe {
        //     let fd = self.socket();
        //     let mut timeout = libc::timeval { tv_sec: 5, tv_usec: 0 };
        //     let mut read_set = mem::uninitialized();
        //     libc::FD_ZERO(&mut read_set);
        //     libc::FD_SET(fd, &mut read_set);
        //     libc::select(fd + 1, &mut read_set, ptr::null_mut(), ptr::null_mut(), &mut timeout);
        //     libc::FD_ISSET(fd, &mut read_set)
        // }
    }
}

#[cfg(target_os = "windows")]
pub(crate) type Socket = winapi::um::winsock2::SOCKET;

pub(crate) fn socket_is_ready(socket: Socket, timeout: Duration) -> Result<bool> {
    #[cfg(target_os = "windows")]
    os::socket_is_ready(socket, timeout)
}
