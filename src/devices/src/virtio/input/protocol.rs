// Copyright 2024 The libkrun Authors. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
//
// Portions derived from virtio-input specification and Linux kernel headers
// Licensed under BSD-3-Clause

//! virtio-input device protocol definitions

use vm_memory::ByteValued;

// virtio-input device configuration selectors
pub const VIRTIO_INPUT_CFG_UNSET: u8 = 0x00;
pub const VIRTIO_INPUT_CFG_ID_NAME: u8 = 0x01;
pub const VIRTIO_INPUT_CFG_ID_SERIAL: u8 = 0x02;
pub const VIRTIO_INPUT_CFG_ID_DEVIDS: u8 = 0x03;
pub const VIRTIO_INPUT_CFG_PROP_BITS: u8 = 0x10;
pub const VIRTIO_INPUT_CFG_EV_BITS: u8 = 0x11;
pub const VIRTIO_INPUT_CFG_ABS_INFO: u8 = 0x12;

// Linux input event types (from linux/input-event-codes.h)
pub const EV_SYN: u16 = 0x00;
pub const EV_KEY: u16 = 0x01;
pub const EV_REL: u16 = 0x02;
pub const EV_ABS: u16 = 0x03;
pub const EV_REP: u16 = 0x14;

// Relative axes
pub const REL_X: u16 = 0x00;
pub const REL_Y: u16 = 0x01;
pub const REL_WHEEL: u16 = 0x08;

// Mouse buttons
pub const BTN_LEFT: u16 = 0x110;
pub const BTN_RIGHT: u16 = 0x111;
pub const BTN_MIDDLE: u16 = 0x112;

// Synchronization events
pub const SYN_REPORT: u16 = 0;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct VirtioInputAbsInfo {
    pub min: u32,
    pub max: u32,
    pub fuzz: u32,
    pub flat: u32,
    pub res: u32,
}

unsafe impl ByteValued for VirtioInputAbsInfo {}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct VirtioInputDevIds {
    pub bustype: u16,
    pub vendor: u16,
    pub product: u16,
    pub version: u16,
}

unsafe impl ByteValued for VirtioInputDevIds {}

#[repr(C)]
#[derive(Copy, Clone)]
pub union VirtioInputConfigUnion {
    pub string: [u8; 128],
    pub bitmap: [u8; 128],
    pub abs: VirtioInputAbsInfo,
    pub ids: VirtioInputDevIds,
}

impl Default for VirtioInputConfigUnion {
    fn default() -> Self {
        VirtioInputConfigUnion {
            bitmap: [0u8; 128],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct VirtioInputConfig {
    pub select: u8,
    pub subsel: u8,
    pub size: u8,
    pub reserved: [u8; 5],
    pub u: VirtioInputConfigUnion,
}

unsafe impl ByteValued for VirtioInputConfig {}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct VirtioInputEvent {
    pub event_type: u16,
    pub code: u16,
    pub value: u32,
}

unsafe impl ByteValued for VirtioInputEvent {}

impl VirtioInputEvent {
    pub fn new(event_type: u16, code: u16, value: u32) -> Self {
        VirtioInputEvent {
            event_type,
            code,
            value,
        }
    }

    pub fn syn_report() -> Self {
        Self::new(EV_SYN, SYN_REPORT, 0)
    }

    pub fn key(code: u16, pressed: bool) -> Self {
        Self::new(EV_KEY, code, if pressed { 1 } else { 0 })
    }

    pub fn rel_motion(axis: u16, value: i32) -> Self {
        Self::new(EV_REL, axis, value as u32)
    }
}

// Bus types for input devices
pub const BUS_PCI: u16 = 0x01;
pub const BUS_VIRTUAL: u16 = 0x06;

// Helper to set a bit in a bitmap
pub fn set_bit(bitmap: &mut [u8], bit: usize) {
    let byte_idx = bit / 8;
    let bit_idx = bit % 8;
    if byte_idx < bitmap.len() {
        bitmap[byte_idx] |= 1 << bit_idx;
    }
}
