//! Extensions for the JSON support.

use crate::history::State;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;

pub fn serialize_os_string<S: Serializer>(string: &OsStr, ser: S) -> Result<S::Ok, S::Error> {
    ser.serialize_str(&String::from_utf8_lossy(string.as_bytes()))
}

pub fn serialize_vec_os_string<S: Serializer>(
    strings: &[OsString],
    ser: S,
) -> Result<S::Ok, S::Error> {
    let mut seq = ser.serialize_seq(Some(strings.len()))?;
    for s in strings {
        let s = String::from_utf8_lossy(s.as_bytes());
        seq.serialize_element(&s)?;
    }
    seq.end()
}

pub fn serialize_state<S: Serializer>(data: &State, ser: S) -> Result<S::Ok, S::Error> {
    let mut map = ser.serialize_map(Some(1))?;

    match data {
        State::Running { start } => {
            map.serialize_entry("running", &MonotonicTime(Timespec(start)))?;
        }

        State::Finished {
            running_time,
            status,
            rusage,
        } => {
            map.serialize_entry(
                "finished",
                &Finished {
                    running_time,
                    status,
                    rusage,
                },
            )?;
        }
    }

    map.end()
}

// Types for fields in `State`.

struct MonotonicTime<'a>(Timespec<'a>);

struct Timespec<'a>(&'a libc::timespec);

struct Finished<'a> {
    running_time: &'a Option<std::time::Duration>,
    status: &'a libc::c_int,
    rusage: &'a libc::rusage,
}

struct Rusage<'a>(&'a libc::rusage);

struct Timeval<'a>(&'a libc::timeval);

// Serializers for C structs.

impl Serialize for MonotonicTime<'_> {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let mut fields = ser.serialize_map(Some(1))?;
        fields.serialize_entry("monotonic", &self.0)?;
        fields.end()
    }
}

impl Serialize for Timespec<'_> {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let mut fields = ser.serialize_map(Some(2))?;
        fields.serialize_entry("tv_sec", &self.0.tv_sec)?;
        fields.serialize_entry("tv_nsec", &self.0.tv_nsec)?;
        fields.end()
    }
}

impl Serialize for Finished<'_> {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let mut fields = ser.serialize_map(None)?;
        if let Some(r) = self.running_time {
            fields.serialize_entry("running_time_secs", &r.as_secs_f64())?;
        }
        fields.serialize_entry("status", self.status)?;
        fields.serialize_entry("resource_usage", &Rusage(self.rusage))?;
        fields.end()
    }
}

impl Serialize for Rusage<'_> {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let mut map = ser.serialize_map(Some(16))?;
        map.serialize_entry("ru_utime", &Timeval(&self.0.ru_utime))?;
        map.serialize_entry("ru_stime", &Timeval(&self.0.ru_stime))?;
        map.serialize_entry("ru_maxrss", &self.0.ru_maxrss)?;
        map.serialize_entry("ru_ixrss", &self.0.ru_ixrss)?;
        map.serialize_entry("ru_idrss", &self.0.ru_idrss)?;
        map.serialize_entry("ru_isrss", &self.0.ru_isrss)?;
        map.serialize_entry("ru_minflt", &self.0.ru_minflt)?;
        map.serialize_entry("ru_majflt", &self.0.ru_majflt)?;
        map.serialize_entry("ru_nswap", &self.0.ru_nswap)?;
        map.serialize_entry("ru_inblock", &self.0.ru_inblock)?;
        map.serialize_entry("ru_oublock", &self.0.ru_oublock)?;
        map.serialize_entry("ru_msgsnd", &self.0.ru_msgsnd)?;
        map.serialize_entry("ru_msgrcv", &self.0.ru_msgrcv)?;
        map.serialize_entry("ru_nsignals", &self.0.ru_nsignals)?;
        map.serialize_entry("ru_nvcsw", &self.0.ru_nvcsw)?;
        map.serialize_entry("ru_nivcsw", &self.0.ru_nivcsw)?;
        map.end()
    }
}

impl Serialize for Timeval<'_> {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let mut fields = ser.serialize_map(Some(2))?;
        fields.serialize_entry("secs", &self.0.tv_sec)?;
        fields.serialize_entry("usecs", &self.0.tv_usec)?;
        fields.end()
    }
}
