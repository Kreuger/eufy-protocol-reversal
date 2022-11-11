use btleplug::api::{bleuuid::BleUuid, Central, CentralEvent, Manager as _, ScanFilter};
use btleplug::api::{BDAddr, CharPropFlags, Characteristic, Peripheral};
use btleplug::platform::{Adapter, Manager};
use futures::stream::StreamExt;
use std::error::Error;
use std::str::FromStr;

async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().nth(0).unwrap()
}

fn dispatch(data: Vec<u8>) {
    let get_time = vec![172, 2, 243, 0, 0, 0, 204];
    let sync_time = vec![172, 2, 242, 0, 0, 0, 0, 204];
    let get_unit = vec![172, 2, 254, 27, 255, 0, 204];
    let set_unit = vec![172, 2, 254, 6, 0, 0, 204];
    if (data.len() >= 8) && (data[0] == 172) && (data[1] == 2) {
        if data.len() == 19 {
            if data[2] != 254 {
                let i = u32::from_le_bytes([data[3], data[4], data[5], data[6]]);
                let i2 = u16::from_be_bytes([data[7], data[8]]);
                let i3 = u16::from_be_bytes([data[9], data[10]]);
                println!("Action_Get_History({},{},{})", i2, i3, i);

                let i = u32::from_le_bytes([data[11], data[12], data[13], data[14]]);
                let i2 = u16::from_be_bytes([data[15], data[16]]);
                let i3 = u16::from_be_bytes([data[17], data[18]]);
                println!("Action_Get_History({},{},{})", i2, i3, i);
            }

            let i = u32::from_le_bytes([data[3], data[4], data[5], data[6]]);
            let i2 = u16::from_be_bytes([data[7], data[8]]);
            let i3 = u16::from_be_bytes([data[9], data[10]]);
            println!("Action_Real_Time_Record({},{},{})", i2, i3, i);
        } else if data.len() == 16 {
            dispatch(data[..8].to_vec());
            dispatch(data[8..].to_vec());
        } else if data.len() == 17 {
            dispatch(data[9..].to_vec());
            dispatch(data[..9].to_vec());
        } else if (data[2] == 254) && (data[3] == 7) && (data[6] == sync_time[7]) {
            println!("Action Sync Time True");
        } else if (data[2] == 254) && (data[3] == 8) && (data[6] == sync_time[7]) {
            println!("Action Sync Time False");
        } else if (data[2] == get_unit[2]) && (data[3] == get_unit[3]) && (data[6] == get_unit[6]) {
            println!("GET_UNIT {}", data[4]);
        } else if (data[2] == set_unit[2]) && (data[3] == set_unit[3]) && (data[6] == set_unit[6]) {
            println!("SET_UNIT {}", data[4] == 254);
        } else if data[6] == 206 {
            println!("Dynamic Weight {}", u16::from_be_bytes([data[2], data[3]]));
        } else if data[6] == 202 {
            println!("Stable Weight {}", u16::from_be_bytes([data[2] , data[3]]));
        } else if data[2] == 253 && data[3] == 0 && data[6] == 203 {
            println!("ACTION_START_IMPEDANCE_MEASUREMENT");
        } else if data[2] == 253 && data[3] == 1 && data[6] == 203 {
            println!(
                "ACTION_IMPEDANCE_MEASUREMENT_RESULT {}",
                u16::from_be_bytes([data[4],data[5]])
            );
        } else if data[2] == 241 && data[7] == 204 {
            println!(
                "ACTION_MEASUREMENT_TIME {}",
                u32::from_le_bytes([data[3] , data[4], data[5] ,data[6]])
            );
        } else if data[2] == 254 && data[3] == 16 && data[6] == 204 {
            println!("Action Measure Finish");
        } else if data[2] == 254 && data[3] == 1 && data[6] == 204 {
            println!("get history start");
        } else if data[2] == 254 && data[3] == 2 && data[6] == 204 {
            println!("get history end");
            println!("Get History Success");
        } else if data[2] == 254 && data[3] == 0 && data[6] == 204 {
            println!("no history record");
        } else if data[2] == 254 && data[3] == 28 && data[4] == 255 && data[6] == 204 {
            println!("ready power off");
        } else if data[2] == 254 && data[3] == 26 && data[6] == 204 {
            println!("power off");
        } else if data[2] == 254 && data[3] == 3 && data[6] == 204 {
            println!("Over Weight");
        } else if data[2] == get_time[2] {
            println!(
                "Get Time {} {} ",
                u32::from_le_bytes([data[3], data[4], data[5], data[6]]),
                u32::from_le_bytes([data[7], data[8], data[9], data[10]])
            );
        }
    }
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
                    perif
                        .subscribe(&Characteristic {
                            // This is one of the characteristics that supply notify, trying to test it out
                            uuid: uuid::Uuid::from_str("4143f6b2530049004700414943415245")?,
                            service_uuid: uuid::Uuid::from_str("4143f6b0530049004700414943415245")?,
                            properties: CharPropFlags::NOTIFY,
                        })
                        .await?;
                    let mut notifs = perif.notifications().await?;
                    while let Some(notif) = notifs.next().await {
                        dispatch(notif.value);
                    }
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
