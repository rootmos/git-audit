use log::{Record, Metadata, Level};
use std::io;
use std::path::Path;

extern crate chrono;
use self::chrono::prelude::{DateTime, Local};

extern crate serde;
use self::serde::ser::{Serialize, Serializer, SerializeStruct};

use std::os::unix::net::UnixDatagram;
pub struct UnixSocketLogger {
    socket: UnixDatagram,
}

struct WrappedLevel(Level);

impl Serialize for WrappedLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let mut l = serializer.serialize_struct("Level", 2)?;
        l.serialize_field("numeric", &(self.0 as usize))?;
        l.serialize_field("label", &self.0)?;
        l.end()
    }
}

struct WrappedRecord<'a> {
    time: DateTime<Local>,
    record: &'a Record<'a>,
}

impl Serialize for WrappedRecord<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let mut s = serializer.serialize_struct("Record", 4)?;
        s.serialize_field("time", &self.time)?;
        s.serialize_field("level", &WrappedLevel(self.record.level()))?;
        s.serialize_field("target", &self.record.target())?;
        s.serialize_field("message", &self.record.args())?;
        s.end()
    }
}

impl UnixSocketLogger {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<UnixSocketLogger> {
        let socket = UnixDatagram::unbound()?;
        socket.connect(path)?;
        Ok(UnixSocketLogger { socket })
    }
}

impl log::Log for UnixSocketLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let time = Local::now();
            let s = serde_json::to_string(&WrappedRecord { time, record, }).unwrap();
            self.socket.send(s.as_bytes()).unwrap();
        }
    }

    fn flush(&self) {}
}
