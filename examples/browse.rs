use env_logger::Env;
use log::{error, info};
// use std::net::ToSocketAddrs;
use astro_dnssd::{BrowseError, ServiceBrowserBuilder, ServiceEventType};
use std::io::ErrorKind;
use std::time::Duration;

fn main() {
    env_logger::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting browser...");
    let browser = ServiceBrowserBuilder::new("_http._tcp").browse();
    match browser {
        Ok(browser) => {
            info!("Browser started!");
            loop {
                match browser.recv_timeout(Duration::from_millis(500)) {
                    Ok(service) => {
                        if service.event_type == ServiceEventType::Added {
                            info!("Service found: {:?}", service);
                        } else {
                            info!("Service left: {}", service.hostname);
                        }
                    }
                    Err(BrowseError::IoError(e)) if e.kind() == ErrorKind::TimedOut => {
                        std::thread::sleep(Duration::from_millis(100));
                    }
                    Err(BrowseError::Timeout) => {
                        std::thread::sleep(Duration::from_millis(100));
                    }
                    Err(e) => {
                        error!("Error receiving browser service: {:?}", e);
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        }
        Err(e) => {
            error!("Error starting browser: {:?}", e);
        }
    }
}
