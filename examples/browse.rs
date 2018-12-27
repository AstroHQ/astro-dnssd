extern crate dnssd_rs;
use std::time::Duration;
use std::thread;
use dnssd_rs::browser::*;

fn main() {
    let mut browser = DNSServiceBrowserBuilder::new("_http._tcp").build().unwrap();
    let _result = browser.start(|interface, error_code, service_name, regtype, reply_domain| {
        if error_code != 0 {
            println!("Error: {}", error_code);
            return;
        }
        println!("Reply: if: {} name: {} type: {} domain: {}", interface, service_name, regtype, reply_domain);
    });
    browser.process_result();
    thread::sleep(Duration::from_secs(10));
    println!("Exiting");
}
