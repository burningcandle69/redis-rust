use super::Redis;
use super::utils::make_io_error;
use crate::resp::RESP;
use std::ops::Add;
use std::time::Duration;

impl Redis {
    pub fn set(&mut self, mut args: Vec<RESP>) -> std::io::Result<()> {
        let mut store = self.store.lock().unwrap();
        let key = args.remove(0).hashable();
        let value = args.remove(0);
        store.kv.insert(key.clone(), value.into());

        if args.len() > 0 {
            let unit = args
                .remove(0)
                .string()
                .ok_or(make_io_error("expected string for unit of time"))?;
            let mut time = args
                .remove(0)
                .string()
                .ok_or(make_io_error("expected expiry time"))?
                .parse()
                .unwrap();
            if unit.to_lowercase() == "ex" {
                time *= 1000;
            }
            let expiry_time = std::time::Instant::now().add(Duration::from_millis(time));
            store.expiry.insert(expiry_time, key);
        }

        let resp: RESP = "OK".into();
        write!(self.io, "{resp}")
    }

    /// Gets value from the key value store
    /// return null bulk string if not found
    pub fn get(&mut self, mut args: Vec<RESP>) -> std::io::Result<()> {
        self.remove_expired();
        let store = self.store.lock().unwrap();

        let key = args.remove(0).hashable();
        if let Some(v) = store.kv.get(&key).and_then(|v| v.string()) {
            write!(self.io, "{v}")
        } else {
            write!(self.io, "{}", RESP::null_bulk_string())
        }
    }

    /// Removes expired keys from the kv store
    /// keys are stored in heap wrt their expiration time
    pub fn remove_expired(&mut self) {
        let mut store = self.store.lock().unwrap();
        while !store.expiry.is_empty() {
            let (t, key) = match store.expiry.pop_first() {
                Some(v) => v,
                None => break,
            };
            if t > std::time::Instant::now() {
                store.expiry.insert(t, key);
                break;
            }
            store.kv.remove(&key);
        }
    }
}
