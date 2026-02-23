use crossbeam_channel::unbounded;
mod graphics;
mod hardware;
use hardware::HeadsetInfo;

fn main() {
    let mut response_buf = [0u8; 64];
    let mut headset = HeadsetInfo::default();

    headset.device = HeadsetInfo::get_device_handle();

    match headset.device {
        Some(_) => println!("device handle found!"),
        None => println!("device handle not found!")
    }

    headset.battery_level = headset.get_battery(&mut response_buf);
    
    if headset.battery_level > 0 {
        println!("current battery percentage: {}%", headset.battery_level);
    } else {
        println!("failed to get battery level!");
    }

}
