// Copyright 2022 Takashi Toyoshima <toyoshim@gmail.com>. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be found
// in the LICENSE file.
use rusb;

pub struct Ch559 {
    handle: Option<rusb::DeviceHandle<rusb::GlobalContext>>,
    ep_in: u8,
    ep_out: u8,
    chip_id: u8,
    version: String,
    sum: u8,
    key_is_reset: bool,
}

impl Ch559 {
    pub fn new() -> Self {
        const VID: u16 = 0x4348;
        const PID: u16 = 0x55e0;
        let mut ch559 = Ch559 {
            handle: rusb::open_device_with_vid_pid(VID, PID),
            ep_in: 0,
            ep_out: 0,
            chip_id: 0,
            version: String::from("unknown"),
            sum: 0,
            key_is_reset: false,
        };
        if ch559.is_connected() {
            if let Err(error) = ch559.initialize() {
                ch559.handle = None;
                println!("{}", error);
            }
        }
        return ch559;
    }

    pub fn is_connected(&self) -> bool {
        match self.handle {
            Some(_) => return true,
            None => return false,
        }
    }

    pub fn erase(&mut self) -> Result<(), String> {
        if let None = &mut self.handle {
            return Err(String::from("invalid handle"));
        }
        if let Err(error) = self.reset_key() {
            return Err(error);
        }
        const ERASE_SIZE: u8 = 60;
        let request = [0xa4, 0x01, 0x00, ERASE_SIZE];
        let mut response: [u8; 6] = [0; 6];
        match self.send_receive(&request, &mut response) {
            Ok(_) => {
                if 0 != response[4] {
                    return Err(String::from("failed to erase"));
                }
                return Ok(());
            }
            Err(string) => return Err(string),
        }
    }

    pub fn erase_data(&mut self) -> Result<(), String> {
        if let None = &mut self.handle {
            return Err(String::from("invalid handle"));
        }
        if let Err(error) = self.reset_key() {
            return Err(error);
        }
        let request = [0xa9, 0x00, 0x00, 0x00];
        let mut response: [u8; 6] = [0; 6];
        match self.send_receive(&request, &mut response) {
            Ok(_) => {
                if 0 != response[4] {
                    return Err(String::from("failed to erase"));
                }
                return Ok(());
            }
            Err(string) => return Err(string),
        }
    }

    fn initialize(&mut self) -> Result<(), String> {
        if let Some(handle) = &mut self.handle {
            let device = handle.device();
            let config = device.config_descriptor(0);
            let config_number;
            let interface_number;
            if let Ok(config) = config {
                config_number = config.number();
                if let Some(interface) = config.interfaces().next() {
                    interface_number = interface.number();
                    if let Some(desc) = interface.descriptors().next() {
                        let mut ep_in_found = false;
                        let mut ep_in_type = rusb::TransferType::Bulk;
                        let mut ep_out_found = false;
                        let mut ep_out_type = rusb::TransferType::Bulk;
                        for ep in desc.endpoint_descriptors() {
                            match ep.direction() {
                                rusb::Direction::In => {
                                    self.ep_in = ep.address();
                                    ep_in_type = ep.transfer_type();
                                    ep_in_found = true;
                                }
                                rusb::Direction::Out => {
                                    self.ep_out = ep.address();
                                    ep_out_type = ep.transfer_type();
                                    ep_out_found = true;
                                }
                            }
                        }
                        if !ep_in_found
                            || !ep_out_found
                            || ep_in_type != rusb::TransferType::Bulk
                            || ep_out_type != rusb::TransferType::Bulk
                        {
                            return Err(String::from("failed to detect EPs"));
                        }
                    }
                } else {
                    return Err(String::from("failed to check interfaces"));
                }
            } else {
                return Err(String::from("failed to check configurations"));
            }
            if let Err(_) = handle.set_active_configuration(config_number) {
                return Err(String::from("failed to activate the target configuration"));
            }
            if let Err(_) = handle.claim_interface(interface_number) {
                return Err(String::from("failed to claim the target interface"));
            }
            let detect_request = [
                0xa1, 0x12, 0x00, 0x59, 0x11, 0x4d, 0x43, 0x55, 0x20, 0x49, 0x53, 0x50, 0x20, 0x26,
                0x20, 0x57, 0x43, 0x48, 0x2e, 0x43, 0x4e,
            ];
            let mut detect_response: [u8; 6] = [0; 6];
            match self.send_receive(&detect_request, &mut detect_response) {
                Ok(_) => {
                    if detect_response[4] != 0x59 {
                        return Err(String::from("failed to receive a valid response on detect"));
                    }
                    self.chip_id = detect_response[4];
                }
                Err(string) => return Err(string + " on detect"),
            }
            let identify_request = [0xa7, 0x02, 0x00, 0x1f, 0x00];
            let mut identify_response: [u8; 30] = [0; 30];
            match self.send_receive(&identify_request, &mut identify_response) {
                Ok(_) => {
                    self.version = format!(
                        "{}.{}{}",
                        identify_response[19], identify_response[20], identify_response[21],
                    );
                }
                Err(string) => return Err(string + " on detect"),
            };

            println!("CH559 Found (BootLoader: v{})", self.version);
            self.sum = identify_response[22]
                .wrapping_add(identify_response[23])
                .wrapping_add(identify_response[24])
                .wrapping_add(identify_response[25]);
            return Ok(());
        } else {
            return Err(String::from("invalid handle"));
        }
    }

    fn reset_key(&mut self) -> Result<(), String> {
        if self.key_is_reset {
            return Ok(());
        }
        let mut request = [0; 0x33];
        request[0] = 0xa3;
        request[1] = 0x30;
        request[2] = 0x00;
        for i in 3..0x33 {
            request[i] = self.sum;
        }
        let mut response = [0; 6];
        match self.send_receive(&request, &mut response) {
            Ok(_) => {
                if response[4] != self.chip_id {
                    return Err(String::from("failed to reset key"));
                }
                self.key_is_reset = true;
                return Ok(());
            }
            Err(error) => return Err(error),
        }
    }

    fn send_receive(&mut self, request: &[u8], response: &mut [u8]) -> Result<(), String> {
        if let Some(handle) = &mut self.handle {
            if let Ok(size) =
                handle.write_bulk(self.ep_out, request, core::time::Duration::new(1, 0))
            {
                if size != request.len() {
                    return Err(String::from("failed to do a bulk write all data"));
                }
            } else {
                return Err(String::from("failed to do a bulk write"));
            }
            match handle.read_bulk(self.ep_in, response, core::time::Duration::new(1, 0)) {
                Ok(_) => return Ok(()),
                Err(error) => {
                    return Err(String::from(format!(
                        "failed to do a bulk read response ({})",
                        error
                    )));
                }
            }
        } else {
            return Err(String::from("invalid handle"));
        }
    }
}
