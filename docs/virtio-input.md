# Virtio-Input Device Support in libkrun

This document describes the virtio-input device implementation in libkrun, which provides keyboard and mouse emulation for GUI workloads.

## Overview

The virtio-input devices enable hardware-accelerated desktop environments via podman with keyboard and mouse support. This implementation is compatible with both macOS and Linux hosts when combined with GPU paravirtualization using Venus.

## Architecture

### Components

1. **Protocol Layer** (`src/devices/src/virtio/input/protocol.rs`)
   - Defines virtio-input protocol structures based on Linux kernel headers
   - Implements event types (keyboard, mouse, relative motion, buttons)
   - Provides helper functions for creating input events

2. **Device Layer** (`src/devices/src/virtio/input/device.rs`)
   - Implements the `Input` virtio device
   - Provides separate keyboard and mouse devices
   - Handles device configuration and event queues
   - Implements the `VirtioDevice` and `Subscriber` traits

3. **Event Handler** (`src/devices/src/virtio/input/event_handler.rs`)
   - Manages event processing for input devices
   - Placeholder for future event queue handling

4. **Integration** (`src/vmm/src/builder.rs`)
   - Integrates input devices into the VMM builder
   - Attaches keyboard and mouse devices to the MMIO bus

## C API

### Enable Virtio-Input

```c
/**
 * Enables virtio-input devices (keyboard and mouse) for the VM.
 *
 * Returns:
 *  Zero on success or a negative error number on failure.
 */
int32_t krun_enable_virtio_input(uint32_t ctx_id);
```

### Event Injection (TODO)

The following API functions are defined but not yet fully implemented:

```c
int32_t krun_inject_keyboard_event(uint32_t ctx_id, uint16_t keycode, uint8_t pressed);
int32_t krun_inject_mouse_button(uint32_t ctx_id, uint16_t button, uint8_t pressed);
int32_t krun_inject_mouse_motion(uint32_t ctx_id, int32_t dx, int32_t dy);
int32_t krun_inject_mouse_wheel(uint32_t ctx_id, int32_t delta);
```

## Usage Example

```rust
use krun_sys::*;

unsafe {
    let ctx = krun_create_ctx();
    
    // Configure GPU with Venus support
    krun_set_gpu_options(
        ctx,
        VIRGLRENDERER_USE_EGL
            | VIRGLRENDERER_VENUS
            | VIRGLRENDERER_RENDER_SERVER
            | VIRGLRENDERER_THREAD_SYNC
            | VIRGLRENDERER_USE_ASYNC_FENCE_CB
    );
    
    // Enable virtio-input devices
    krun_enable_virtio_input(ctx);
    
    // Add display
    krun_add_display(ctx, 1920, 1080);
    
    // Configure workload and start VM
    // ...
    
    krun_start_enter(ctx);
}
```

## Device Configuration

### Keyboard Device

- **Device Type**: virtio-input (ID 18)
- **Device Name**: "virtio-keyboard"
- **Supported Events**:
  - `EV_KEY`: Key press/release events
  - `EV_REP`: Key repeat events
  - `EV_SYN`: Synchronization events

### Mouse Device

- **Device Type**: virtio-input (ID 18)
- **Device Name**: "virtio-mouse"
- **Supported Events**:
  - `EV_KEY`: Button press/release (left, right, middle)
  - `EV_REL`: Relative motion (X, Y axes, wheel)
  - `EV_SYN`: Synchronization events

## Guest Support

For proper operation in the guest:

1. The guest kernel must have virtio-input support enabled:
   ```
   CONFIG_VIRTIO_INPUT=y
   ```

2. The guest should have the appropriate input drivers loaded:
   ```
   modprobe virtio_input
   ```

3. Input devices will appear as `/dev/input/eventX` devices

4. Desktop environments should automatically detect and use these devices

## Implementation Status

### Completed

- ✅ Virtio-input protocol definitions
- ✅ Keyboard device implementation
- ✅ Mouse device implementation  
- ✅ Device integration into VMM
- ✅ C API for enabling devices
- ✅ Example usage in gui_vm

### TODO

- ⏳ Event injection mechanism for host-to-guest input
- ⏳ Event queue processing
- ⏳ Support for absolute positioning (tablet device)
- ⏳ Support for multi-touch events

## Platform Support

- **Linux Host**: Fully supported
- **macOS Host**: Fully supported with HVF

## Testing

To test the virtio-input implementation:

1. Build libkrun with the changes:
   ```bash
   make
   ```

2. Run the gui_vm example with a desktop environment:
   ```bash
   cargo run --example gui_vm --release -- \
       --display 1920x1080 \
       --root-dir /path/to/rootfs \
       /usr/bin/startxfce4
   ```

3. Verify that keyboard and mouse devices appear in the guest:
   ```bash
   # Inside the guest
   ls -l /dev/input/event*
   cat /proc/bus/input/devices
   ```

## References

- [Virtio Input Device Specification](https://docs.oasis-open.org/virtio/virtio/v1.1/cs01/virtio-v1.1-cs01.html#x1-3390008)
- [Linux Kernel virtio-input driver](https://github.com/torvalds/linux/blob/master/drivers/virtio/virtio_input.c)
- [Venus GPU Support Discussion](https://github.com/ericcurtin/krunkit/issues/3)
