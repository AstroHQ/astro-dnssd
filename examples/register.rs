extern crate dnssd_rs;
// use std::time::Duration;
// use std::thread;
use dnssd_rs::register::*;
use dnssd_rs::txt::*;

fn main() {
    println!("Registering service...");
    let mut txt = TXTRecord::new();
    let _ = txt.set_value("s", "open");
    let mut service = DNSServiceBuilder::new("_rust._tcp")
        .with_port(2048)
        .with_name("MyRustService")
        .with_txt_record(txt)
        .build().unwrap();
    let _result = service.register(|_, error, _, _, _| {
        println!("Registered: {}", error);
    });
    loop {
        // if service.has_data() {
            println!("Has data!");
            service.process_result();
        // }
    }
    // service.process_result();
    // thread::sleep(Duration::from_secs(10));
    // println!("Exiting");
}
