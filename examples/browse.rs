extern crate dnssd_rs;
use std::time::Duration;
use std::thread;
use dnssd_rs::browser::*;

fn main() {
    let mut browser = ServiceBrowserBuilder::new("_http._tcp").build().unwrap();
    let _result = browser.start(|result| {
        match result {
            Ok(service) => println!("Reply: if: {} name: {} type: {} domain: {}", service.interface_index, service.name, service.regtype, service.domain),
            Err(e) => println!("Error: {:?}", e),
        }
    });
    browser.process_result();
    thread::sleep(Duration::from_secs(10));
    println!("Exiting");
}
