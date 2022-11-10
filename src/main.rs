use btleplug::api::{bleuuid::BleUuid, Central, CentralEvent, Manager as _, ScanFilter};
use btleplug::api::{BDAddr, Characteristic, Peripheral, CharPropFlags};
use btleplug::platform::{Adapter, Manager, PeripheralId};
use futures::stream::StreamExt;
use std::error::Error;
use std::str::FromStr;

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().nth(0).unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await?;

    // get the first bluetooth adapter
    // connect to the adapter
    let central = get_central(&manager).await;

    // Each adapter has an event stream, we fetch via events(),
    // simplifying the type, this will return what is essentially a
    // Future<Result<Stream<Item=CentralEvent>>>.
    let mut events = central.events().await?;

    // start scanning for devices
    central.start_scan(ScanFilter::default()).await?;

    // Print based on whatever the event receiver outputs. Note that the event
    // receiver blocks, so in a real program, this should be run in its own
    // thread (not task, as this library does not yet use async channels).
    while let Some(event) = events.next().await {
        match event {
            CentralEvent::DeviceDiscovered(id) => {
                let perif = central.peripheral(&id).await?;
                let baddr = perif.properties().await?.unwrap().address;
                if BDAddr::from_str_delim("03:B3:EC:94:40:F1") == Ok(baddr) {
                    println!("DETECTED");
                    perif.connect().await?;
                }
            }
            CentralEvent::DeviceConnected(id) => {
                let perif = central.peripheral(&id).await?;
                let baddr = perif.properties().await?.unwrap().address;
                if BDAddr::from_str_delim("03:B3:EC:94:40:F1") == Ok(baddr) {
                    println!("CONNECTED");
                    perif.discover_services().await?;
                    println!("Characteristics: {:#?}", perif.characteristics());
                    println!("Services: {:#?}", perif.services());
                    perif.subscribe(&Characteristic {
                        // This is one of the characteristics that supply notify, trying to test it out
                        uuid: uuid::Uuid::from_str("4143f6b2530049004700414943415245")?,
                        service_uuid:  uuid::Uuid::from_str("4143f6b0530049004700414943415245")?,
                        properties: CharPropFlags::NOTIFY,
                    }).await?;
                }
            }
            CentralEvent::DeviceDisconnected(id) => {
                println!("DeviceDisconnected: {:?}", id);
            }
            CentralEvent::ManufacturerDataAdvertisement {
                id,
                manufacturer_data,
            } => {
                let perif = central.peripheral(&id).await?;
                let baddr = perif.properties().await?.unwrap().address;
                if BDAddr::from_str_delim("03:B3:EC:94:40:F1") == Ok(baddr) {
                    println!(
                        "ManufacturerDataAdvertisement: {:?}, {:?}",
                        id, manufacturer_data
                    );
                }
            }
            CentralEvent::ServiceDataAdvertisement { id, service_data } => {
                println!("ServiceDataAdvertisement: {:?}, {:?}", id, service_data);
            }
            CentralEvent::ServicesAdvertisement { id, services } => {
                let services: Vec<String> =
                    services.into_iter().map(|s| s.to_short_string()).collect();
                println!("ServicesAdvertisement: {:?}, {:?}", id, services);
            }
            _ => {}
        }
    }
    Ok(())
}
