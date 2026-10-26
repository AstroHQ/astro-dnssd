//! Advertises or browses a service over AWDL as well as the usual interfaces, printing each found service's addresses.
//!
//! `cargo run --example p2p -- register _test._udp 44565` on one device,
//! `cargo run --example p2p -- browse _test._udp` on another

#[cfg(target_vendor = "apple")]
fn main() {
    use astro_dnssd::{BrowseError, DNSServiceBuilder, ServiceBrowserBuilder, ServiceEventType};
    use env_logger::Env;
    use std::time::Duration;

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["register", regtype, port] => {
            let port = port.parse().expect("port must be a number");
            let _service = DNSServiceBuilder::new(regtype, port)
                .with_peer_to_peer(true)
                .register()
                .expect("register failed");
            println!("Registered {regtype} on {port} with peer-to-peer, ctrl-c to stop");
            loop {
                std::thread::sleep(Duration::from_secs(60));
            }
        }
        ["browse", regtype] => {
            let browser = ServiceBrowserBuilder::new(regtype)
                .with_peer_to_peer(true)
                .browse()
                .expect("browse failed");
            println!("Browsing {regtype} with peer-to-peer, ctrl-c to stop");
            loop {
                match browser.recv_timeout(Duration::from_millis(500)) {
                    Ok(service) if service.event_type == ServiceEventType::Added => {
                        let addrs = service.resolve_socket_addrs(Duration::from_secs(5));
                        println!(
                            "+ {} on {} (interface {:?}): {addrs:?}",
                            service.name, service.hostname, service.interface_index
                        );
                    }
                    Ok(service) => println!(
                        "- {} (interface {:?})",
                        service.name, service.interface_index
                    ),
                    Err(BrowseError::Timeout) => {}
                    Err(e) => println!("browse error: {e:?}"),
                }
            }
        }
        _ => println!("usage: p2p register <regtype> <port> | p2p browse <regtype>"),
    }
}

#[cfg(not(target_vendor = "apple"))]
fn main() {
    println!("peer-to-peer is only on Apple platforms");
}
