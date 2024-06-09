use std::time::Duration;

#[derive(Debug, speedy::Readable, speedy::Writable)]
pub struct Event {
	pub time: Duration,
	pub event_type: u16,
	pub event_code: u16,
	pub value: i32,
}

#[derive(Debug, speedy::Readable, speedy::Writable)]
pub struct Capabilities {
	pub event_types: Vec<i32>,
	pub event_codes: Vec<(i32, u16)>,
}

#[derive(Debug, speedy::Readable, speedy::Writable)]
pub struct EventList {
	#[speedy(constant_prefix = "MACR{O}")]
	pub capabilities: Capabilities,

	pub events: Vec<Event>,
}

impl EventList {
	pub fn new(c: Capabilities) -> Self {
		Self {
			capabilities: c,
			events: vec![],
		}
	}
	pub fn push(&mut self, event: Event) {
		self.events.push(event);
	}
	pub fn len(&self) -> usize {
		self.events.len()
	}
}
