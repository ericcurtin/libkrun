// Copyright 2024 The libkrun Authors. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! virtio-input device implementation

use std::cmp;
use std::io::Write;
use std::result;
use std::sync::{Arc, Mutex};

use crossbeam_channel::{unbounded, Sender};
use utils::eventfd::EventFd;
use vm_memory::{ByteValued, Bytes, GuestMemoryMmap};

use super::super::Queue as VirtQueue;
use super::defs::uapi;
use super::protocol::*;
use crate::virtio::{ActivateResult, DeviceState, InterruptTransport, VirtioDevice};

const QUEUE_SIZE: u16 = 256;
const QUEUE_SIZES: &[u16] = &[QUEUE_SIZE, QUEUE_SIZE];

#[derive(Debug)]
pub enum Error {
    EventFd(std::io::Error),
}

pub type Result<T> = result::Result<T, Error>;

pub enum InputDeviceType {
    Keyboard,
    Mouse,
}

pub struct Input {
    device_type: InputDeviceType,
    device_name: String,
    name: String,
    serial: String,
    queues: Vec<VirtQueue>,
    queue_events: Vec<EventFd>,
    avail_features: u64,
    acked_features: u64,
    device_state: DeviceState,
    config: VirtioInputConfig,
    event_sender: Sender<VirtioInputEvent>,
    event_buffer: Arc<Mutex<Vec<VirtioInputEvent>>>,
}

impl Input {
    pub fn new_keyboard() -> Result<Self> {
        Self::new(InputDeviceType::Keyboard, "virtio-keyboard", "keyboard-1")
    }

    pub fn new_mouse() -> Result<Self> {
        Self::new(InputDeviceType::Mouse, "virtio-mouse", "mouse-1")
    }

    fn new(device_type: InputDeviceType, name: &str, serial: &str) -> Result<Self> {
        let mut queues = Vec::new();
        let mut queue_events = Vec::new();

        for &size in QUEUE_SIZES {
            queues.push(VirtQueue::new(size));
            queue_events.push(EventFd::new(utils::eventfd::EFD_NONBLOCK).map_err(Error::EventFd)?);
        }

        let avail_features = 1u64 << uapi::VIRTIO_F_VERSION_1;
        
        let (event_sender, _event_receiver) = unbounded();
        let event_buffer = Arc::new(Mutex::new(Vec::new()));

        Ok(Input {
            device_type,
            device_name: format!("virtio-input-{}", name),
            name: name.to_string(),
            serial: serial.to_string(),
            queues,
            queue_events,
            avail_features,
            acked_features: 0,
            device_state: DeviceState::Inactive,
            config: VirtioInputConfig::default(),
            event_sender,
            event_buffer,
        })
    }

    fn get_config(&self, select: u8, subsel: u8) -> VirtioInputConfig {
        let mut config = VirtioInputConfig::default();
        config.select = select;
        config.subsel = subsel;

        match select {
            VIRTIO_INPUT_CFG_ID_NAME => {
                let name_bytes = self.name.as_bytes();
                config.size = cmp::min(name_bytes.len(), 128) as u8;
                unsafe {
                    config.u.string[..config.size as usize]
                        .copy_from_slice(&name_bytes[..config.size as usize]);
                }
            }
            VIRTIO_INPUT_CFG_ID_SERIAL => {
                let serial_bytes = self.serial.as_bytes();
                config.size = cmp::min(serial_bytes.len(), 128) as u8;
                unsafe {
                    config.u.string[..config.size as usize]
                        .copy_from_slice(&serial_bytes[..config.size as usize]);
                }
            }
            VIRTIO_INPUT_CFG_ID_DEVIDS => {
                config.size = std::mem::size_of::<VirtioInputDevIds>() as u8;
                unsafe {
                    config.u.ids = VirtioInputDevIds {
                        bustype: BUS_VIRTUAL,
                        vendor: 0x1af4, // Red Hat vendor ID for virtio
                        product: match self.device_type {
                            InputDeviceType::Keyboard => 1,
                            InputDeviceType::Mouse => 2,
                        },
                        version: 1,
                    };
                }
            }
            VIRTIO_INPUT_CFG_EV_BITS => {
                match self.device_type {
                    InputDeviceType::Keyboard => {
                        if subsel == EV_KEY as u8 {
                            // For keyboard, we support all key codes
                            config.size = 128;
                            unsafe {
                                // Set all bits for key events (simplified - real implementation
                                // should only set supported keys)
                                config.u.bitmap.fill(0xff);
                            }
                        } else if subsel == EV_REP as u8 {
                            config.size = 1;
                            unsafe {
                                config.u.bitmap[0] = 0x03; // REP_DELAY and REP_PERIOD
                            }
                        }
                    }
                    InputDeviceType::Mouse => {
                        if subsel == EV_KEY as u8 {
                            config.size = 24; // Enough for mouse buttons
                            unsafe {
                                set_bit(&mut config.u.bitmap, BTN_LEFT as usize);
                                set_bit(&mut config.u.bitmap, BTN_RIGHT as usize);
                                set_bit(&mut config.u.bitmap, BTN_MIDDLE as usize);
                            }
                        } else if subsel == EV_REL as u8 {
                            config.size = 2;
                            unsafe {
                                set_bit(&mut config.u.bitmap, REL_X as usize);
                                set_bit(&mut config.u.bitmap, REL_Y as usize);
                                set_bit(&mut config.u.bitmap, REL_WHEEL as usize);
                            }
                        }
                    }
                }
                if config.size > 0 {
                    // Always indicate EV_SYN support
                    let mut bitmap = vec![0u8; 128];
                    set_bit(&mut bitmap, EV_SYN as usize);
                    match self.device_type {
                        InputDeviceType::Keyboard => {
                            set_bit(&mut bitmap, EV_KEY as usize);
                            set_bit(&mut bitmap, EV_REP as usize);
                        }
                        InputDeviceType::Mouse => {
                            set_bit(&mut bitmap, EV_KEY as usize);
                            set_bit(&mut bitmap, EV_REL as usize);
                        }
                    }
                    
                    if subsel == 0 {
                        config.size = 1;
                        unsafe {
                            config.u.bitmap[..1].copy_from_slice(&bitmap[..1]);
                        }
                    }
                }
            }
            _ => {
                config.size = 0;
            }
        }

        config
    }

    pub fn send_event(&self, event: VirtioInputEvent) {
        let mut buffer = self.event_buffer.lock().unwrap();
        buffer.push(event);
        // TODO: Signal the event queue
    }
}

impl VirtioDevice for Input {
    fn device_type(&self) -> u32 {
        uapi::VIRTIO_ID_INPUT
    }

    fn device_name(&self) -> &str {
        &self.device_name
    }

    fn queues(&self) -> &[VirtQueue] {
        &self.queues
    }

    fn queues_mut(&mut self) -> &mut [VirtQueue] {
        &mut self.queues
    }

    fn queue_events(&self) -> &[EventFd] {
        &self.queue_events
    }

    fn avail_features(&self) -> u64 {
        self.avail_features
    }

    fn acked_features(&self) -> u64 {
        self.acked_features
    }

    fn set_acked_features(&mut self, acked_features: u64) {
        self.acked_features = acked_features;
    }

    fn read_config(&self, offset: u64, mut data: &mut [u8]) {
        let config_slice = self.config.as_slice();
        let config_len = config_slice.len() as u64;
        if offset >= config_len {
            error!("Failed to read config space");
            return;
        }
        if let Some(end) = offset.checked_add(data.len() as u64) {
            data.write_all(&config_slice[offset as usize..cmp::min(end, config_len) as usize])
                .unwrap();
        }
    }

    fn write_config(&mut self, offset: u64, data: &[u8]) {
        let config_len = std::mem::size_of::<VirtioInputConfig>() as u64;
        if offset >= config_len {
            error!("Failed to write config space");
            return;
        }

        // Parse the config write to determine what the guest is requesting
        if offset == 0 && data.len() >= 2 {
            let select = data[0];
            let subsel = data[1];
            self.config = self.get_config(select, subsel);
        }
    }

    fn is_activated(&self) -> bool {
        self.device_state.is_activated()
    }

    fn activate(&mut self, mem: GuestMemoryMmap, interrupt: InterruptTransport) -> ActivateResult {
        self.device_state = DeviceState::Activated(mem, interrupt);
        Ok(())
    }
}
