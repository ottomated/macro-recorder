use core::slice;
use std::{
	ffi::{c_void, CString},
	mem,
	path::Path,
	ptr, thread,
	time::Duration,
};

use anyhow::Context;
use evdev_rs::enums::{int_to_event_type, EventType};
use nix::{
	errno::Errno,
	fcntl,
	libc::{self, gettimeofday, timeval, uinput_user_dev},
	sys::stat,
};
use speedy::Readable;
use uinput::Event;
use uinput_sys::{
	input_event, ui_dev_create, ui_set_absbit, ui_set_evbit, ui_set_ffbit, ui_set_keybit,
	ui_set_ledbit, ui_set_mscbit, ui_set_relbit, ui_set_sndbit, ui_set_swbit,
};

use crate::common::{Capabilities, EventList};

pub fn play(input: &Path) -> anyhow::Result<()> {
	let events = EventList::read_from_file(input).context("Failed to read input")?;
	// let fd = create_uinput_device(&events.capabilities)?;
	let mut device = uinput::default()
		.context("Failed to get uinput")?
		.name("Macro Playback")
		.context("Failed to set name")?
		.event(Event::All)
		.context("Failed to enable all events")?
		.create()
		.context("Failed to create uinput device")?;

	for i in 0..events.len() {
		let event = &events.events[i];

		device.write(
			event.event_type.into(),
			event.event_code.into(),
			event.value,
		)?;
		device.synchronize()?;

		// println!("{} {}", event.event_type, event.event_code);
		// let mut out_event = input_event {
		// 	time: timeval {
		// 		tv_sec: 0,
		// 		tv_usec: 0,
		// 	},
		// 	kind: event.event_type,
		// 	code: event.event_code,
		// 	value: event.value,
		// };
		// unsafe {
		// 	gettimeofday(&mut out_event.time, ptr::null_mut());
		// 	let ptr = &event as *const _ as *const c_void;
		// 	let size = mem::size_of_val(&out_event);
		// 	Errno::result(nix::libc::write(fd, ptr, size)).context("Failed to play event")?;
		// }
		let next_event = events.events.get(i + 1);
		if let Some(next_event) = next_event {
			let sleep_amount = next_event.time - event.time;
			thread::sleep(sleep_amount);
		}
	}

	Ok(())
}

// pub fn create_uinput_device(capabilities: &Capabilities) -> anyhow::Result<i32> {
// 	let fd = fcntl::open("/dev/uinput", fcntl::OFlag::O_WRONLY, stat::Mode::empty())?;

// 	let mut def: uinput_user_dev = unsafe { std::mem::zeroed() };
// 	let name = CString::new("Macro Playback")?;
// 	let name_bytes = name.as_bytes_with_nul();

// 	def.name[..name_bytes.len()].copy_from_slice(unsafe { std::mem::transmute(name_bytes) });
// 	for event_type in capabilities.event_types.iter() {
// 		evdev_rs::UInputDevice::create_from_device(device)
// 		Errno::result(ui_set_evbit(fd, *event_type))
// 	}

// 	for (event_type_raw, code) in capabilities.inner.iter() {
// 		let event_type = int_to_event_type(*event_type_raw as u32).context("Invalid event_type")?;
// 		println!("{event_type_raw} {event_type} {code}");
// 		let code = *code as i32;
// 		unsafe {
// 			Errno::result(ui_set_evbit(fd, *event_type_raw))
// 				.with_context(|| format!("Failed to set event bit for {event_type}"))?;
// 		}
// 		let ret = unsafe {
// 			match event_type {
// 				EventType::EV_SYN => continue,
// 				EventType::EV_KEY => ui_set_keybit(fd, code),
// 				EventType::EV_REL => ui_set_relbit(fd, code),
// 				EventType::EV_ABS => ui_set_absbit(fd, code),
// 				EventType::EV_MSC => ui_set_mscbit(fd, code),
// 				EventType::EV_SW => ui_set_swbit(fd, code),
// 				EventType::EV_LED => ui_set_ledbit(fd, code),
// 				EventType::EV_SND => ui_set_sndbit(fd, code),
// 				EventType::EV_REP => continue,
// 				EventType::EV_FF => ui_set_ffbit(fd, code),
// 				_ => todo!(),
// 			}
// 		};
// 		Errno::result(ret).with_context(|| format!("Failed to set event bit for {event_type}"))?;
// 	}

// 	unsafe {
// 		let ptr = &def as *const _ as *const c_void;
// 		let size = mem::size_of_val(&def);
// 		Errno::result(libc::write(fd, ptr, size)).context("Failed to write uinput device")?;
// 		Errno::result(ui_dev_create(fd)).context("Failed to create uinput device")?;
// 	}

// 	Ok(fd)
// }
