// use astro_dnssd::browser::*;
use env_logger::Env;
use log::{error, info};
use std::net::ToSocketAddrs;

fn main() {
    // env_logger::from_env(Env::default().default_filter_or("trace")).init();
    // info!("Starting browser...");
    // let mut browser = ServiceBrowserBuilder::new("_http._tcp").build();
    // let _result = browser.start(|result| match result {
    //     Ok(mut service) => {
    //         let event = match service.event_type {
    //             ServiceEventType::Added => "Added",
    //             ServiceEventType::Removed => "Removed",
    //         };
    //         info!(
    //             "{}: if: {} name: {} type: {} domain: {}",
    //             event, service.interface_index, service.name, service.regtype, service.domain
    //         );
    //         let results = service.resolve();
    //         for r in results.unwrap() {
    //             let status = r.txt_record.as_ref().unwrap().get("status");
    //             let addrs_iter = r.to_socket_addrs().unwrap();
    //             for addr in addrs_iter {
    //                 info!("Addr: {}", addr);
    //             }
    //             info!("Resolved service: {:?} status: {:?}", r, status);
    //         }
    //     }
    //     Err(e) => error!("Error: {:?}", e),
    // });
    // loop {
    //     match browser.has_data(std::time::Duration::from_millis(500)) {
    //         Ok(true) => {
    //             info!("Has data!");
    //             browser.process_result();
    //             info!("===== Data done processing");
    //         }
    //         Ok(false) => {
    //             info!("No data yet...");
    //             std::thread::sleep(std::time::Duration::from_millis(500));
    //         }
    //         Err(e) => {
    //             error!("Error checking for data: {}", e);
    //             break;
    //         }
    //     }
    // }
    //
    // // thread::sleep(Duration::from_secs(10));
    // // println!("Exiting");
}
