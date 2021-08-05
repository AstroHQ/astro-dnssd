// use astro_dnssd::register::DNSServiceBuilder;
use astro_dnssd::DNSServiceBuilder;
use env_logger::Env;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    env_logger::from_env(Env::default().default_filter_or("trace")).init();
    println!("Registering service...");
    // let mut txt: HashMap<String, String> = HashMap::new();
    // let _ = txt.insert("status".into(), "open".into());
    let service = DNSServiceBuilder::new("_http._tcp", 8080)
        .with_key_value("status".into(), "open".into())
        .register();

    {
        match service {
            Ok(service) => {
                println!("Registered... waiting 20s");
                sleep(Duration::from_secs(20));
                println!("Dropping... {:?}", service);
            }
            Err(e) => {
                println!("Error registering: {:?}", e);
            }
        }
    }
    log::info!("Drop should have happened");
    sleep(Duration::from_secs(5));
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
