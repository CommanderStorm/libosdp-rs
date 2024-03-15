//
// Copyright (c) 2023-2024 Siddharth Chandrasekaran <sidcha.dev@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

mod common;
type Result<T> = core::result::Result<T, libosdp::OsdpError>;

use std::{sync::MutexGuard, thread, time};

use libosdp::{
    Channel, ControlPanel, OsdpCommand, OsdpCommandBuzzer, OsdpEvent, OsdpEventCardRead,
    PeripheralDevice,
};

use crate::common::{
    device::CpDevice, device::PdDevice, memory_channel::MemoryChannel, threadbus::ThreadBus,
};

fn send_command(mut cp: MutexGuard<'_, ControlPanel>, command: OsdpCommand) -> Result<()> {
    cp.send_command(0, command)
}

fn notify_event(mut pd: MutexGuard<'_, PeripheralDevice>, event: OsdpEvent) -> Result<()> {
    pd.notify_event(event)
}

#[test]
fn test_thread_bus_channel() -> Result<()> {
    let mut a = ThreadBus::new("conn-0");
    let mut b = a.clone();
    let mut c = a.clone();

    let buf_write = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut buf_read = [0; 100];
    assert_eq!(a.write(&buf_write)?, buf_write.len());
    assert_eq!(b.read(&mut buf_read)?, buf_write.len());
    assert_eq!(c.read(&mut buf_read)?, buf_write.len());
    Ok(())
}

#[test]
fn test_commands() -> Result<()> {
    common::setup();
    let (cp_bus, pd_bus) = MemoryChannel::new();
    let pd = PdDevice::new(Box::new(pd_bus))?;
    let cp = CpDevice::new(Box::new(cp_bus))?;

    thread::sleep(time::Duration::from_secs(2));
    loop {
        if pd.get_device().is_sc_active() {
            break;
        }
        thread::sleep(time::Duration::from_secs(1));
    }

    let command = OsdpCommand::Buzzer(OsdpCommandBuzzer::default());
    send_command(cp.get_device(), command.clone())?;
    let cmd_rx = pd.receiver.recv().unwrap();
    assert_eq!(cmd_rx, command, "Buzzer command check failed");

    let event = OsdpEvent::CardRead(OsdpEventCardRead::new_ascii(vec![0x55, 0xAA]));
    notify_event(pd.get_device(), event.clone())?;
    assert_eq!(
        cp.receiver.recv().unwrap(),
        (0_i32, event),
        "Card read event check failed"
    );

    Ok(())
}
