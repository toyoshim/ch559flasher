// Copyright 2022 Takashi Toyoshima <toyoshim@gmail.com>. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be found
// in the LICENSE file.
use std::fs::File;
use std::io::{Read, Write};
use thiserror::Error;

mod progress_bar;
use crate::ch559::progress_bar::ProgressBar;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to erase")]
    Erase,
    #[error("IO error")]
    Io(#[from] std::io::Error),
    #[error("failed to do a bulk write all data")]
    BulkWriteAll,
    #[error("failed to do a bulk write")]
    BulkWrite,
    #[error("failed to do a bulk read response ({0})")]
    BulkRead(rusb::Error),
    #[error("failed to reset key")]
    ResetKey,
    #[error("unexpected EOF")]
    Eof,
    #[error("failed to detect EPs")]
    DetectEp,
    #[error("failed to check interfaces")]
    CheckInterface,
    #[error("failed to check configurations")]
    CheckConfiguration,
    #[error("failed to activate the target configuration")]
    ActivateConfiguration,
    #[error("failed to claim the target interface")]
    ClaimInterface,
    #[error("failed to receive a valid response on detect")]
    InvalidResponse,
    #[error("{0} on detect")]
    OnDetect(Box<Error>),
    #[error("read size is too large")]
    TooLargeReadSize,
    #[error("failed to read")]
    Read,
    #[error("failed to flash")]
    Flash,
    #[error("failed to verify")]
    Verify,
    #[error("not a regular file")]
    InvalidFile,
    #[error("file size should be 0x400")]
    FileSize,
    #[error("file size is too large for data")]
    TooLargeDataSize,
    #[error("file size is too large for code")]
    TooLargeCodeSize,
    #[error("failed to initialize")]
    Initialize(Box<Error>),
    #[error("CH559 Not Found")]
    NotFound,
}

pub struct Ch559 {
    handle: rusb::DeviceHandle<rusb::GlobalContext>,
    ep_in: u8,
    ep_out: u8,
    chip_id: u8,
    version: String,
    sum: u8,
    key_is_reset: bool,
    seed: i64,
}

impl Ch559 {
    pub fn new() -> Result<Self, Error> {
        const VID: u16 = 0x4348;
        const PID: u16 = 0x55e0;
        if let Some(handle) = rusb::open_device_with_vid_pid(VID, PID) {
            let mut ch559 = Ch559 {
                handle,
                ep_in: 0,
                ep_out: 0,
                chip_id: 0,
                version: String::from("unknown"),
                sum: 0,
                key_is_reset: false,
                seed: 1,
            };
            ch559
                .initialize()
                .map_err(|e| Error::Initialize(Box::new(e)))?;
            Ok(ch559)
        } else {
            Err(Error::NotFound)
        }
    }

    pub fn set_seed(&mut self, seed: i64) {
        self.seed = seed;
    }

    pub fn erase(&mut self) -> Result<(), Error> {
        self.reset_key()?;
        const ERASE_SIZE: u8 = 60;
        let request = [0xa4, 0x01, 0x00, ERASE_SIZE];
        let mut response: [u8; 6] = [0; 6];
        self.send_receive(&request, &mut response)?;
        if 0 != response[4] {
            return Err(Error::Erase);
        }
        Ok(())
    }

    pub fn erase_data(&mut self) -> Result<(), Error> {
        self.reset_key()?;
        let request = [0xa9, 0x00, 0x00, 0x00];
        let mut response: [u8; 6] = [0; 6];
        self.send_receive(&request, &mut response)?;
        if 0 != response[4] {
            return Err(Error::Erase);
        }
        Ok(())
    }

    pub fn read_data(&mut self, filename: &String) -> Result<(), Error> {
        let mut file = File::create(filename)?;
        self.reset_key()?;
        let mut bar = ProgressBar::new(0x400);
        for offset in (0..0x400).step_by(0x38) {
            bar.progress(offset);
            let remaining_size = 0x400 - offset;
            let size: usize = if remaining_size > 0x38 {
                0x38
            } else {
                remaining_size
            };
            let mut response: Vec<u8> = vec![0; size];
            self.read_data_in_range(offset as u16, &mut response)?;
            file.write_all(&response)?;
            bar.progress(offset + size);
        }
        Ok(())
    }

    pub fn write(
        &mut self,
        filename: &String,
        write: bool,
        data_region: bool,
        fullfill: bool,
    ) -> Result<(), Error> {
        let mut file = File::open(filename)?;
        let metadata = file.metadata()?;
        if !metadata.is_file() {
            return Err(Error::InvalidFile);
        }
        let file_length = metadata.len() as usize;
        if data_region {
            if !fullfill && 0x400 != file_length {
                return Err(Error::FileSize);
            }
            if file_length > 0x400 {
                return Err(Error::TooLargeDataSize);
            }
        } else {
            if file_length > 0xf400 {
                return Err(Error::TooLargeCodeSize);
            }
            if file_length > 0xf000 {
                println!("code will run over data region as file size is larger than 0xF000");
            }
        }
        self.reset_key()?;
        let length = if fullfill {
            if data_region {
                0x400
            } else if file_length > 0xf000 {
                0xf400
            } else {
                0xf000
            }
        } else {
            file_length
        };
        let mut bar = ProgressBar::new(length);
        let mut rand = srand::Rand::new(srand::RngSource::new(self.seed));
        for offset in (0..length).step_by(0x38) {
            bar.progress(offset);
            let remaining_size = length - offset;
            let size: usize = if remaining_size > 0x38 {
                0x38
            } else {
                remaining_size
            };
            let mut data: Vec<u8> = vec![0; size];
            let read_size = if offset > file_length {
                0
            } else if offset + size > file_length {
                file_length - offset
            } else {
                size
            };
            if 0 != read_size {
                let size = file.read(&mut data)?;
                if read_size != size {
                    return Err(Error::Eof);
                }
            }
            if read_size != size {
                for i in read_size..size {
                    data[i] = rand.uint32() as u8;
                }
            }
            self.write_verify_in_range(offset as u16, &data, write, data_region)?;
            bar.progress(offset + size);
        }
        Ok(())
    }

    fn initialize(&mut self) -> Result<(), Error> {
        let device = self.handle.device();
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
                        return Err(Error::DetectEp);
                    }
                }
            } else {
                return Err(Error::CheckInterface);
            }
        } else {
            return Err(Error::CheckConfiguration);
        }
        if self.handle.set_active_configuration(config_number).is_err() {
            return Err(Error::ActivateConfiguration);
        }
        if self.handle.claim_interface(interface_number).is_err() {
            return Err(Error::ClaimInterface);
        }
        let detect_request = [
            0xa1, 0x12, 0x00, 0x59, 0x11, 0x4d, 0x43, 0x55, 0x20, 0x49, 0x53, 0x50, 0x20, 0x26,
            0x20, 0x57, 0x43, 0x48, 0x2e, 0x43, 0x4e,
        ];
        let mut detect_response: [u8; 6] = [0; 6];
        self.send_receive(&detect_request, &mut detect_response)
            .map_err(|e| Error::OnDetect(Box::new(e)))?;
        if detect_response[4] != 0x59 {
            return Err(Error::InvalidResponse);
        }
        self.chip_id = detect_response[4];
        let identify_request = [0xa7, 0x02, 0x00, 0x1f, 0x00];
        let mut identify_response: [u8; 30] = [0; 30];
        self.send_receive(&identify_request, &mut identify_response)
            .map_err(|e| Error::OnDetect(Box::new(e)))?;
        self.version = format!(
            "{}.{}{}",
            identify_response[19], identify_response[20], identify_response[21],
        );

        println!("CH559 Found (BootLoader: v{})", self.version);
        self.sum = identify_response[22]
            .wrapping_add(identify_response[23])
            .wrapping_add(identify_response[24])
            .wrapping_add(identify_response[25]);
        Ok(())
    }

    fn reset_key(&mut self) -> Result<(), Error> {
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
        self.send_receive(&request, &mut response)?;
        if response[4] != self.chip_id {
            return Err(Error::ResetKey);
        }
        self.key_is_reset = true;
        Ok(())
    }

    fn send_receive(&mut self, request: &[u8], response: &mut [u8]) -> Result<(), Error> {
        let size = self
            .handle
            .write_bulk(self.ep_out, request, core::time::Duration::new(1, 0))
            .map_err(|_| Error::BulkWrite)?;
        if size != request.len() {
            return Err(Error::BulkWriteAll);
        }
        self.handle
            .read_bulk(self.ep_in, response, core::time::Duration::new(1, 0))
            .map_err(Error::BulkRead)?;
        Ok(())
    }

    // `addr` is an offset from 0xF000 (DATA_FLASH_ADDR)
    // reset_key() should be called beforehand.
    fn read_data_in_range(&mut self, addr: u16, buffer: &mut [u8]) -> Result<(), Error> {
        if buffer.len() > 0x38 {
            return Err(Error::TooLargeReadSize);
        }
        let request = [
            0xab,
            0x00,
            0x00,
            addr as u8,
            (addr >> 8) as u8,
            0x00,
            0x00,
            buffer.len() as u8,
        ];
        let mut response: Vec<u8> = vec![0; buffer.len() + 6];
        self.send_receive(&request, &mut response)?;
        if 0 != response[4] {
            return Err(Error::Read);
        }
        buffer.copy_from_slice(&response[6..(buffer.len() + 6)]);
        Ok(())
    }

    // `addr` is an offset from 0xF000 (DATA_FLASH_ADDR) if `data_region` is true.
    // reset_key() should be called beforehand.
    fn write_verify_in_range(
        &mut self,
        addr: u16,
        data: &[u8],
        write: bool,
        data_region: bool,
    ) -> Result<(), Error> {
        if data.len() > 0x38 {
            return Err(Error::TooLargeReadSize);
        }
        let write_command = if data_region { 0xaa } else { 0xa5 };
        let length = (data.len() + 7) & !7;
        let mut request: Vec<u8> = Vec::with_capacity(8 + length);
        let address = if data_region && !write {
            addr + 0xF000
        } else {
            addr
        };
        request.push(if write { write_command } else { 0xa6 });
        request.push((length + 5) as u8);
        request.push(0);
        request.push(address as u8);
        request.push((address >> 8) as u8);
        request.push(0);
        request.push(0);
        request.push(length as u8);
        for i in 0..length {
            if i < data.len() {
                request.push(data[i]);
            } else {
                request.push(0xff);
            }
            if 7 == (i & 7) {
                request[8 + i] ^= self.chip_id;
            }
        }
        let mut response: [u8; 6] = [0; 6];
        self.send_receive(&request, &mut response)?;
        if 0 != response[4] {
            let err = if write { Error::Flash } else { Error::Verify };
            return Err(err);
        }
        Ok(())
    }
}
