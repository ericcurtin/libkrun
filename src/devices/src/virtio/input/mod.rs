// Copyright 2024 The libkrun Authors. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! virtio-input device implementation

pub mod protocol;
pub mod device;
pub mod event_handler;

pub use device::Input;
pub use protocol::VirtioInputEvent;

mod defs {
    pub mod uapi {
        /// The device conforms to the virtio spec version 1.0.
        pub const VIRTIO_F_VERSION_1: u32 = 32;
        pub const VIRTIO_ID_INPUT: u32 = 18;
    }
}
