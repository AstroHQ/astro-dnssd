//! Registers a service and changes its TXT record a few times. Watch it with `dns-sd -B _http._tcp` (it shouldn't
//! go away and come back) and `dns-sd -L update-txt _http._tcp` (the record should change)
use astro_dnssd::DNSServiceBuilder;
use env_logger::Env;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    let mut service = DNSServiceBuilder::new("_http._tcp", 8080)
        .with_name("update-txt")
        .with_key_value("count".into(), "0".into())
        .register()
        .expect("failed to register");
    for count in 1..=3 {
        sleep(Duration::from_secs(3));
        let txt = HashMap::from([("count".to_owned(), count.to_string())]);
        println!("Updating TXT to count={count}");
        service.update_txt_record(txt).expect("failed to update");
    }
    sleep(Duration::from_secs(3));
    println!("Clearing TXT");
    service
        .update_txt_record(HashMap::new())
        .expect("failed to clear");
    sleep(Duration::from_secs(3));
}
