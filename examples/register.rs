// use astro_dnssd::register::DNSServiceBuilder;
use astro_dnssd::{DNSService, TxtRecord};
use env_logger::Env;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    env_logger::from_env(Env::default().default_filter_or("trace")).init();
    println!("Registering service...");
    let mut txt = TxtRecord::new();
    let _ = txt.insert("status", "open");
    let service = DNSService {
        regtype: "_LunaDisplay._tcp".to_string(),
        name: None,
        domain: None,
        host: None,
        port: 44554,
        txt: Some(txt),
    };
    {
        match service.register() {
            Ok(service) => {
                sleep(Duration::from_secs(10));
                println!("Dropping... {:?}", service);
            }
            Err(e) => {
                println!("Error registering: {:?}", e);
            }
        }
    }

    // let mut service = DNSServiceBuilder::new("_http._tcp")
    //     .with_port(2048)
    //     .with_name("MyRustService")
    //     .with_txt_record(txt)
    //     .build()
    //     .unwrap();
    // let _result = service.register(|reply| match reply {
    //     Ok(reply) => println!("Successful reply: {:?}", reply),
    //     Err(e) => println!("Error registering: {:?}", e),
    // });
    // loop {
    //     service.process_result();
    // }
}
