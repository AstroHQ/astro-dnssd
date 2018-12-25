extern crate dnssd_rs;
use std::time::Duration;
use std::thread;
use dnssd_rs::*;

fn main() {
    println!("Registering service...");
    let mut service = DNSServiceBuilder::new("_rust._tcp").with_port(2048).with_name("MyRustService").build().unwrap();
    let _result = service.register(|_, error, _, _, _| {
        println!("Registered: {}", error);
    });
    service.process_result();
    thread::sleep(Duration::from_secs(10));
    println!("Exiting");
}
