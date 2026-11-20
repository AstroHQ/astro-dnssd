//! ENG-6284: the registration's processing thread, driven without a live mDNS daemon. A socketpair stands in for the
//! daemon's socket: closing its peer is the daemon going away (as after an iPad sleeps), and `is_alive()` must say so,
//! while dropping the service is a requested shutdown and must not count as a loss

use super::*;
use crate::ffi::apple::{kDNSServiceErr_ServiceNotRunning, kDNSServiceErr_Unknown};
use std::os::raw::c_int;
use std::time::Instant;

const WAIT: Duration = Duration::from_secs(5);

fn socketpair() -> (c_int, c_int) {
    let mut fds = [0 as c_int; 2];
    let r = unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, fds.as_mut_ptr()) };
    assert_eq!(
        r,
        0,
        "socketpair failed: {}",
        std::io::Error::last_os_error()
    );
    (fds[0], fds[1])
}

fn close(fd: c_int) {
    unsafe { libc::close(fd) };
}

fn write_byte(fd: c_int) {
    let b = [1u8];
    let n = unsafe { libc::write(fd, b.as_ptr() as *const c_void, 1) };
    assert_eq!(n, 1, "write failed: {}", std::io::Error::last_os_error());
}

/// What `DNSServiceProcessResult` does with the daemon's socket: reads a reply, and fails once the daemon is gone
fn process_like_dnssd(fd: c_int) -> impl FnMut() -> DNSServiceErrorType + Send + 'static {
    move || {
        let mut buf = [0u8; 64];
        let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut c_void, buf.len()) };
        if n > 0 {
            kDNSServiceErr_NoError
        } else {
            kDNSServiceErr_ServiceNotRunning
        }
    }
}

/// A registered service whose processing thread runs `process` on `fd`, with no DNSServiceRef behind it
fn service_on<F>(fd: c_int, process: F) -> (RegisteredDnsService, std::thread::JoinHandle<()>)
where
    F: FnMut() -> DNSServiceErrorType + Send + 'static,
{
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let alive = Arc::new(AtomicBool::new(true));
    let thread = spawn_processing_thread(fd, process, shutdown_flag.clone(), alive.clone());
    let service = RegisteredDnsService {
        service: Arc::new(Mutex::new(ServiceRef::new(null_mut(), null_mut()))),
        shutdown_flag,
        alive,
    };
    (service, thread)
}

fn wait_until(f: impl Fn() -> bool) -> bool {
    let start = Instant::now();
    while start.elapsed() < WAIT {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    f()
}

#[test]
fn alive_while_processing_replies() {
    let (fd, peer) = socketpair();
    let (service, thread) = service_on(fd, process_like_dnssd(fd));
    for _ in 0..5 {
        write_byte(peer);
    }
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        service.is_alive(),
        "registration reported dead while the daemon socket is fine"
    );
    drop(service);
    thread.join().unwrap();
    close(peer);
    close(fd);
}

#[test]
fn daemon_hangup_marks_registration_dead() {
    let (fd, peer) = socketpair();
    let (service, thread) = service_on(fd, process_like_dnssd(fd));
    assert!(service.is_alive());
    close(peer); // the daemon side goes away, as on wake
    assert!(
        wait_until(|| !service.is_alive()),
        "processing thread lost the daemon socket but is_alive() still says true"
    );
    thread.join().unwrap();
    drop(service);
    close(fd);
}

#[test]
fn process_error_marks_registration_dead() {
    let (fd, peer) = socketpair();
    let (service, thread) = service_on(fd, || kDNSServiceErr_Unknown);
    write_byte(peer);
    assert!(
        wait_until(|| !service.is_alive()),
        "DNSServiceProcessResult failed but is_alive() still says true"
    );
    thread.join().unwrap();
    drop(service);
    close(peer);
    close(fd);
}

#[test]
fn dropping_the_service_is_not_a_loss() {
    let (fd, peer) = socketpair();
    let (service, thread) = service_on(fd, process_like_dnssd(fd));
    let alive = service.alive.clone();
    drop(service);
    thread.join().unwrap();
    assert!(
        alive.load(Ordering::Acquire),
        "a requested shutdown was reported as a lost registration"
    );
    close(peer);
    close(fd);
}
