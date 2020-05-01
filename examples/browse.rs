use astro_dnssd::browser::*;
use env_logger::Env;
use log::{error, info};
use std::net::ToSocketAddrs;

fn main() {
    env_logger::from_env(Env::default().default_filter_or("trace")).init();
    info!("Starting browser...");
    let mut browser = ServiceBrowserBuilder::new("_http._tcp").build().unwrap();
    let _result = browser.start(|result| match result {
        Ok(mut service) => {
            let event = match service.event_type {
                ServiceEventType::Added => "Added",
                ServiceEventType::Removed => "Removed",
            };
            info!(
                "{}: if: {} name: {} type: {} domain: {}",
                event, service.interface_index, service.name, service.regtype, service.domain
            );
            let results = service.resolve();
            for r in results.unwrap() {
                let status = r.txt_record.as_ref().unwrap().get("status");
                let addrs_iter = r.to_socket_addrs().unwrap();
                for addr in addrs_iter {
                    info!("Addr: {}", addr);
                }
                info!("Resolved service: {:?} status: {:?}", r, status);
            }
        }
        Err(e) => error!("Error: {:?}", e),
    });
    loop {
        // if browser.has_data() {
        //     println!("Has data!");
        browser.process_result();
        // }
    }

    // thread::sleep(Duration::from_secs(10));
    // println!("Exiting");
}
