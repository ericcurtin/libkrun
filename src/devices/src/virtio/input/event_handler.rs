// Copyright 2024 The libkrun Authors. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! Event handler for virtio-input device

use polly::event_manager::{EventManager, Subscriber};
use utils::epoll::{EpollEvent, EventSet};

/// Event handler for the virtio-input device
pub struct InputEventHandler {
    // Placeholder for future implementation
}

impl InputEventHandler {
    pub fn new() -> Self {
        InputEventHandler {}
    }
}

impl Subscriber for InputEventHandler {
    fn process(&mut self, event: &EpollEvent, _evmgr: &mut EventManager) {
        let _source = event.fd();
        let event_set = event.event_set();

        if event_set != EventSet::IN {
            warn!("Input: unexpected event_set: {:?}", event_set);
            return;
        }

        debug!("Input event handler process called");
    }

    fn interest_list(&self) -> Vec<EpollEvent> {
        // Return empty list for now as we don't have any file descriptors to monitor yet
        Vec::new()
    }
}
