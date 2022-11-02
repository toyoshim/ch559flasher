// Copyright 2022 Takashi Toyoshima <toyoshim@gmail.com>. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be found
// in the LICENSE file.
use rusb;

pub struct Ch559 {
    handle: Option<rusb::DeviceHandle<rusb::GlobalContext>>,
    ep_in: u8,
    ep_out: u8,
}

impl Ch559 {
    pub fn new() -> Self {
        const VID: u16 = 0x4348;
        const PID: u16 = 0x55e0;
        let mut ch559 = Ch559 {
            handle: rusb::open_device_with_vid_pid(VID, PID),
            ep_in: 0,
            ep_out: 0,
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
                        let mut ep_out_found = false;
                        for ep in desc.endpoint_descriptors() {
                            match ep.direction() {
                                rusb::Direction::In => {
                                    self.ep_in = ep.number();
                                    ep_in_found = true;
                                }
                                rusb::Direction::Out => {
                                    self.ep_out = ep.number();
                                    ep_out_found = true;
                                }
                            }
                        }
                        if !ep_in_found || !ep_out_found {
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
            // TODO: detect
            // TODO: identify
        }
        return Ok(());
    }
}
