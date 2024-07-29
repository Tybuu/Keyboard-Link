use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

use hidapi::{DeviceInfo, HidApi, HidDevice};

fn main() {
    let (tx, rx) = mpsc::sync_channel(65);

    let mut api;
    match HidApi::new() {
        Ok(res) => api = res,
        Err(e) => panic!("{}", e),
    }

    let mut left_dev = open(&mut api, 0xa55, 1);

    let mut right_dev = open(&mut api, 0x727, 0);

    let api = Arc::new(Mutex::new(api));
    let api_clone = api.clone();
    thread::spawn(move || loop {
        loop {
            let mut buf = [0u8; 65];
            buf[1] = 5;
            match right_dev.read(&mut buf[2..65]) {
                Ok(_) => tx.send(buf).expect("Writing Channel failed"),
                Err(_) => right_dev = open(&mut api_clone.lock().unwrap(), 0x727, 0),
            };
        }
    });
    loop {
        let buf = rx.recv().expect("Reading channel failed");
        match left_dev.write(&buf) {
            Ok(_) => {}
            Err(_) => left_dev = open(&mut api.lock().unwrap(), 0xa55, 1),
        }
    }
}

fn open(api: &mut HidApi, pid: u16, i_num: i32) -> HidDevice {
    loop {
        api.refresh_devices().unwrap();
        if let Some(dev) = api
            .device_list()
            .find(|dev: &&DeviceInfo| dev.interface_number() == i_num && dev.product_id() == pid)
        {
            loop {
                let real_dev = dev.open_device(api);
                if real_dev.is_ok() {
                    return real_dev.unwrap();
                }
            }
        }
    }
}
