// See the "macOS permissions note" in README.md before running this on macOS
// Big Sur or later.

mod connect_device;

use btleplug::api::{
    bleuuid::uuid_from_u16, Central, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Manager, Peripheral};
use rand::{thread_rng, Rng};
use std::error::Error;
use std::time::Duration;
use uuid::Uuid;

const LIGHT_CHARACTERISTIC_UUID: Uuid = uuid_from_u16(0x2A5F);
use tokio::time;

async fn find_light(central: &Adapter) -> Option<Peripheral> {
    for p in central.peripherals().await.unwrap() {
        if p.properties()
            .await
            .unwrap()
            .unwrap()
            .local_name
            .iter()
            .any(|name| name.to_lowercase().contains("choicemmed"))
        {
            return Some(p);
        }
    }
    None
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let manager = Manager::new().await.unwrap();

    // get the first bluetooth adapter
    let central = manager
        .adapters()
        .await
        .expect("Unable to fetch adapter list.")
        .into_iter()
        .nth(0)
        .expect("Unable to find adapters.");

    // start scanning for devices
    central.start_scan(ScanFilter::default()).await?;
    // instead of waiting, you can use central.events() to get a stream which will
    // notify you of new devices, for an example of that see examples/event_driven_discovery.rs
    time::sleep(Duration::from_secs(2)).await;

    // find the device we're interested in
    let light = find_light(&central).await.expect("No lights found");

    // connect to the device
    light.connect().await?;

    // discover services and characteristics
    light.discover_services().await?;

    // find the characteristic we want
    let chars = light.characteristics();
    let cmd_char = chars
        .iter()
        .find(|c| c.uuid == LIGHT_CHARACTERISTIC_UUID)
        .expect("Unable to find characterics");

    // dance party
    for _ in 0..20 {
        let res = light.read(&cmd_char).await?;
        println!("Current result = {:?}", res);
        time::sleep(Duration::from_millis(200)).await;
    }
    Ok(())
}