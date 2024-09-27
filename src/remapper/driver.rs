use super::machine::Machine;
use super::types::{EvKeyEvent, KeyEventType};
use crate::mapping::{self, *};
use anyhow::Result;
use anyhow::*;
use evdev_rs::{Device, DeviceWrapper, GrabMode, InputEvent, ReadFlag, TimeVal, UInputDevice};
use std::path::Path;
use std::path::PathBuf;

pub fn run_forever(device_path: PathBuf, mappings: &Vec<mapping::Mapping>) -> Result<()> {
    let mut devices: EvdevDevices = EvdevDevices::create_and_grab_devices(device_path)?;
    devices.enable_key_codes_in_mapping(mappings)?;
    log::info!("Going into read loop");
    let mut machine: Machine = Machine::new(mappings);
    loop {
        let (status, event) = devices
            .input
            .next_event(ReadFlag::NORMAL | ReadFlag::BLOCKING)?;
        match status {
            evdev_rs::ReadStatus::Success => {
                // We'll only be intercepting EV_KEY events and passing them to the machine for processing.
                if let EventCode::EV_KEY(ref key) = event.event_code {
                    log::trace!("IN {:?}", event);
                    let event_type = KeyEventType::from_value(event.value);
                    let converted_events_to_write: Vec<EvKeyEvent> = machine.insert(
                        EvKeyEvent {
                            time: event.time,
                            ev_key: key.clone(),
                            key_event_type: event_type,
                        }
                    );
                    for event in converted_events_to_write {
                        log::trace!("OUT: {:?}", event);
                        devices.output.write_event(&event.as_input_event())?;
                    }
                    devices.generate_sync_event(&event.time)?;
                } else {
                    log::trace!("PASSTHRU {:?}", event);
                    devices.output.write_event(&event)?;
                }
            }
            evdev_rs::ReadStatus::Sync => bail!("ReadStatus::Sync!"),
        }
    }
}

struct EvdevDevices {
    input: Device,
    output: UInputDevice,
}

fn enable_key_code(input: &mut Device, key: KeyCode) -> Result<()> {
    input
        .enable(EventCode::EV_KEY(key.clone()))
        .context(format!("enable key {:?}", key))?;
    Ok(())
}

impl EvdevDevices {
    fn create_and_grab_devices<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let f: std::fs::File =
            std::fs::File::open(path).context(format!("opening {}", path.display()))?;
        let mut input = Device::new_from_file(f)
            .with_context(|| format!("failed to create new Device from file {}", path.display()))?;

        input.set_name(&format!("evremap Virtual input for {}", path.display()));

        let output = UInputDevice::create_from_device(&input)
            .context(format!("creating UInputDevice from {}", path.display()))?;

        input
            .grab(GrabMode::Grab)
            .context(format!("grabbing exclusive access on {}", path.display()))?;

        Ok(Self { input, output })
    }

    fn enable_key_codes_in_mapping(&mut self, mappings: &Vec<mapping::Mapping>) -> Result<()> {
        // Ensure that any remapped keys are supported by the generated output device
        for map in mappings {
            match map {
                Mapping::Remap { output, .. } => {
                    for o in output {
                        enable_key_code(&mut self.input, o.clone())?;
                    }
                }
            }
        }
        return Ok(());
    }

    fn generate_sync_event(&self, time: &TimeVal) -> Result<()> {
        self.output.write_event(&InputEvent::new(
            time,
            &EventCode::EV_SYN(evdev_rs::enums::EV_SYN::SYN_REPORT),
            0,
        ))?;
        Ok(())
    }
}
