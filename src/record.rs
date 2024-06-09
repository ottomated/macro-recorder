use std::{
	fs::File,
	io::{BufWriter, Write as _},
	path::Path,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	time::{Duration, SystemTime},
};

use anyhow::Context;
use evdev_rs::{
	util::event_code_to_int, Device, DeviceWrapper, EventCodeIterator, EventTypeIterator, ReadFlag,
	TimeVal,
};
use speedy::Writable;

use crate::common::{Capabilities, Event, EventList};

const COUNTDOWN: u32 = 3;

pub fn record(device: &Device, output: &Path) -> anyhow::Result<()> {
	let capabilities = get_device_capabilities(device);

	let output_file = File::create(output).context("Can't open output file")?;

	print!("(CTRL-C to end) Starting recording in");
	let _ = std::io::stdout().flush();

	for i in (0..COUNTDOWN).rev() {
		if i > 0 {
			print!(" {},", i + 1);
		} else {
			print!(" 1");
		}
		let _ = std::io::stdout().flush();
		std::thread::sleep(std::time::Duration::from_secs(1));
	}
	println!("\nRecording started");
	let mut start_time = None;

	let mut events = EventList::new(capabilities);

	let running = Arc::new(AtomicBool::new(true));
	let r = running.clone();
	ctrlc::set_handler(move || {
		r.store(false, Ordering::SeqCst);
	})
	.context("Error setting Ctrl-C handler")?;

	while running.load(Ordering::SeqCst) {
		let Ok(ev) = device.next_event(ReadFlag::NORMAL).map(|val| val.1) else {
			continue;
		};

		let event_time: Duration = match start_time {
			Some(start) => subtract_timeval(ev.time, start).unwrap_or_else(|| {
				println!("Event time is before start time");
				Duration::from_secs(0)
			}),
			None => {
				start_time = Some(ev.time);
				Duration::from_secs(0)
			}
		};
		match ev.event_code {
			evdev_rs::enums::EventCode::EV_KEY(_)
			| evdev_rs::enums::EventCode::EV_REL(_)
			| evdev_rs::enums::EventCode::EV_ABS(_) => {}
			_ => {
				continue;
			}
		}
		let (event_type, event_code) = event_code_to_int(&ev.event_code);
		println!("{event_time:?} {} {}", ev.event_code, ev.value);
		events.push(Event {
			time: event_time,
			event_type: event_type as u16,
			event_code: event_code as u16,
			value: ev.value,
		});
	}
	println!("\nFinished recording ({} events)", events.len());
	println!("Writing to {}", output.display());
	events
		.write_to_stream(BufWriter::new(output_file))
		.context("Failed to write output")?;
	Ok(())
}

fn subtract_timeval(x: TimeVal, y: TimeVal) -> Option<Duration> {
	let x: SystemTime = x.try_into().ok()?;
	let y: SystemTime = y.try_into().ok()?;
	x.duration_since(y).ok()
}

fn get_device_capabilities(device: &Device) -> Capabilities {
	let mut event_types = vec![];
	let mut event_codes = vec![];

	for event_type in EventTypeIterator::new().filter(|t| device.has(*t)) {
		event_types.push(event_type as i32);
		for code in EventCodeIterator::new(&event_type).filter(|c| device.has(*c)) {
			let (_, code) = event_code_to_int(&code);
			event_codes.push((event_type as i32, code as u16));
		}
	}

	Capabilities {
		event_types,
		event_codes,
	}
}

fn get_data(fd: i32, type_: i32, size: i32) -> Result<Vec<u8>, nix::errno::Errno> {
	let mut data = vec![0u8; size as usize];
	unsafe {
		nix::errno::Errno::result(nix::libc::ioctl(
			fd,
			nix::request_code_read!(b'E', (0x20 + type_), size) as nix::sys::ioctl::ioctl_num_type,
			data.as_mut_ptr(),
		))?;
	}
	Ok(data)
}
