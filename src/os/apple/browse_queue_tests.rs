//! Browse under load, without a live mDNS daemon: drives `browse_callback` the way `DNSServiceProcessResult` does and
//! checks no event is lost, the callback never blocks, and `recv_timeout` keeps to its timeout

use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const EVENTS: usize = 1_000;
const REGTYPE: &str = "_test._tcp.";
const DOMAIN: &str = "local.";

fn name(i: usize) -> String {
    format!("service {}", i)
}

/// Calls the browse callback like mDNSResponder would for one service, returning how long the call took
fn fire_callback(context: *mut c_void, i: usize, flags: ffi::DNSServiceFlags) -> Duration {
    let name = CString::new(name(i)).unwrap();
    let regtype = CString::new(REGTYPE).unwrap();
    let domain = CString::new(DOMAIN).unwrap();
    let start = Instant::now();
    unsafe {
        browse_callback(
            ptr::null_mut(),
            flags,
            7,
            kDNSServiceErr_NoError,
            name.as_ptr(),
            regtype.as_ptr(),
            domain.as_ptr(),
            context,
        );
    }
    start.elapsed()
}

fn added() -> ffi::DNSServiceFlags {
    ffi::kDNSServiceFlagsAdd as _
}

fn resolved_for(service: &DiscoveredService) -> ResolvedService {
    ResolvedService {
        full_name: format!("{}.{}{}", service.name, service.regtype, service.domain),
        hostname: "host.local.".into(),
        port: 1234,
        txt_record: None,
        interface_index: 7,
    }
}

/// 1,000 discoveries in one burst with a consumer far slower than the callback: every one arrives, in order, and no
/// callback call waits on the consumer. Before this fix the callback used `try_send` into a 10 slot channel and
/// dropped everything past the 10th
#[test]
fn callback_keeps_every_event_and_never_blocks_with_slow_consumer() {
    let (tx, rx) = callback_queue::<Result<DiscoveredService>>();
    let context = Box::into_raw(Box::new(tx));
    let consumer = std::thread::spawn(move || {
        let mut names = Vec::new();
        while let Ok(result) = rx.recv() {
            std::thread::sleep(Duration::from_micros(200));
            names.push(result.expect("no errors").name);
        }
        names
    });

    let mut slowest = Duration::ZERO;
    for i in 0..EVENTS {
        slowest = slowest.max(fire_callback(context as _, i, added()));
    }
    // draining at 200µs each takes >= 200ms, so a callback waiting on the consumer would show here
    assert!(
        slowest < Duration::from_millis(50),
        "browse callback blocked for {:?}",
        slowest
    );

    drop(unsafe { Box::from_raw(context) });
    let names = consumer.join().unwrap();
    assert_eq!(names.len(), EVENTS, "browse callback lost events");
    assert_eq!(names, (0..EVENTS).map(name).collect::<Vec<_>>());
}

/// The whole browse pipeline (callback, resolver, browser's channel) with a slow resolver and a slow app: every
/// discovery comes out resolved
#[test]
fn pipeline_delivers_every_event_with_slow_resolver_and_consumer() {
    let (tx, rx) = callback_queue::<Result<DiscoveredService>>();
    let (final_tx, final_rx) = channel::<Result<Service>>();
    let context = Box::into_raw(Box::new(tx));
    let resolver = std::thread::spawn(move || {
        resolver_loop(rx, final_tx, |service| {
            std::thread::sleep(Duration::from_micros(100));
            Ok(vec![resolved_for(service)])
        })
    });

    let mut slowest = Duration::ZERO;
    for i in 0..EVENTS {
        slowest = slowest.max(fire_callback(context as _, i, added()));
    }
    assert!(
        slowest < Duration::from_millis(50),
        "browse callback blocked for {:?}",
        slowest
    );

    let mut names = Vec::new();
    while names.len() < EVENTS {
        let service = recv_until(
            &final_rx,
            Duration::from_secs(5),
            |_| Ok(false),
            || unreachable!(),
        )
        .expect("resolved service before timeout");
        assert_eq!(service.port, 1234);
        std::thread::sleep(Duration::from_micros(100));
        names.push(service.name);
    }
    assert_eq!(names, (0..EVENTS).map(name).collect::<Vec<_>>());

    drop(unsafe { Box::from_raw(context) });
    resolver.join().unwrap();
}

/// A removed service has nothing left to answer a resolve, so it's passed on without one instead of waiting out
/// the resolve timeout in front of every other event
#[test]
fn removed_services_skip_resolve() {
    let (tx, rx) = callback_queue::<Result<DiscoveredService>>();
    let (final_tx, final_rx) = channel::<Result<Service>>();
    let resolves = Arc::new(AtomicUsize::new(0));
    let counted = resolves.clone();
    let context = Box::into_raw(Box::new(tx));
    fire_callback(context as _, 1, 0);
    drop(unsafe { Box::from_raw(context) });

    resolver_loop(rx, final_tx, move |_| {
        counted.fetch_add(1, Ordering::SeqCst);
        Err(BrowseError::Timeout)
    });
    let service = final_rx.recv().unwrap().expect("removed service");
    assert_eq!(service.name, name(1));
    assert_eq!(service.event_type, ServiceEventType::Removed);
    assert_eq!(resolves.load(Ordering::SeqCst), 0);
}

fn service(i: usize) -> Service {
    Service {
        name: name(i),
        regtype: REGTYPE.into(),
        interface_index: Some(7),
        domain: DOMAIN.into(),
        event_type: ServiceEventType::Added,
        hostname: "host.local.".into(),
        port: 1234,
        txt_record: None,
    }
}

/// A service that's already queued comes back without waiting on the daemon socket first
#[test]
fn recv_returns_queued_service_without_waiting_on_socket() {
    let (tx, rx) = channel();
    tx.send(Ok(service(1))).unwrap();
    let start = Instant::now();
    let got = recv_until(
        &rx,
        Duration::from_secs(5),
        |_| panic!("waited on the socket with a service queued"),
        || unreachable!(),
    )
    .unwrap();
    assert_eq!(got.name, name(1));
    assert!(start.elapsed() < Duration::from_millis(50));
}

/// With nothing arriving, recv_timeout gives up after `timeout`, not after waiting on the socket and then the
/// channel for `timeout` each
#[test]
fn recv_timeout_waits_at_most_timeout() {
    let (_tx, rx) = channel::<Result<Service>>();
    let timeout = Duration::from_millis(200);
    let start = Instant::now();
    let result = recv_until(
        &rx,
        timeout,
        |slice| {
            std::thread::sleep(slice);
            Ok(false)
        },
        || unreachable!(),
    );
    let elapsed = start.elapsed();
    assert!(matches!(result, Err(BrowseError::Timeout)), "{:?}", result);
    assert!(elapsed >= timeout, "returned early: {:?}", elapsed);
    assert!(
        elapsed < timeout + Duration::from_millis(100),
        "waited {:?} for a {:?} timeout",
        elapsed,
        timeout
    );
}

/// Every reply waiting on the socket is processed in one call, not one per recv_timeout
#[test]
fn recv_processes_every_waiting_reply() {
    let (_tx, rx) = channel::<Result<Service>>();
    let waiting = std::cell::Cell::new(5);
    let processed = std::cell::Cell::new(0);
    let _ = recv_until(
        &rx,
        Duration::ZERO,
        |_| Ok(waiting.get() > 0),
        || {
            waiting.set(waiting.get() - 1);
            processed.set(processed.get() + 1);
            kDNSServiceErr_NoError
        },
    );
    assert_eq!(processed.get(), 5);
}
