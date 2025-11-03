use crate::redis::Redis;
use crate::redis::utils::make_io_error;
use crate::redis::value::{StreamEntry, StreamEntryID, Value};
use crate::resp::RESP;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

impl Redis {
    pub fn xadd(&mut self, mut args: Vec<RESP>) -> std::io::Result<()> {
        let key = args.remove(0).hashable();
        let mut store = self.store.lock().unwrap();
        let stream = store
            .kv
            .entry(key)
            .or_insert(Value::new_stream())
            .stream_mut()
            .unwrap();
        let id = args
            .remove(0)
            .string()
            .ok_or(make_io_error("expected string id"))?;
        let mut data = HashMap::new();
        while args.len() > 0 {
            let key = args.remove(0).hashable();
            let value = args.remove(0);
            data.insert(key, value);
        }

        let id = if id == "*" {
            StreamEntryID::new()
        } else if id.contains("*") {
            let time: usize = id.split("-").nth(0).unwrap().parse().unwrap();
            let mut id = StreamEntryID::with_time(time);
            if let Some(x) = stream.last()
                && x.id.time == time
            {
                id.sqn = x.id.sqn + 1;
            }
            id
        } else {
            StreamEntryID::explicit(id)
        };

        let entry = StreamEntry { id, data };
        if id == (StreamEntryID { time: 0, sqn: 0 }) {
            let resp =
                RESP::SimpleError("ERR The ID specified in XADD must be greater than 0-0".into());
            write!(self.io, "{resp}")
        } else if stream.is_empty() || &entry > stream.last().unwrap() {
            stream.push(entry);
            let resp: RESP = id.to_string().into();
            write!(self.io, "{resp}")
        } else {
            let resp = RESP::SimpleError(
                "ERR The ID specified in XADD is equal or smaller than the target stream top item"
                    .into(),
            );
            write!(self.io, "{resp}")
        }
    }

    pub fn xrange(&mut self, mut args: Vec<RESP>) -> std::io::Result<()> {
        let key = args.remove(0).hashable();
        let mut store = self.store.lock().unwrap();
        let stream = store
            .kv
            .entry(key)
            .or_insert(Value::new_stream())
            .stream_mut()
            .unwrap();

        let start = args.remove(0).string().unwrap();
        let end = args.remove(0).string().unwrap();

        let start = if start == "-" {
            0
        } else {
            let id = StreamEntryID::implicit(start);
            stream.partition_point(|x| x.id < id)
        };

        let end = if end == "+" {
            stream.len()
        } else {
            let id = StreamEntryID::implicit(end);
            stream.partition_point(|x| x.id <= id)
        };

        let res = stream.get(start..end).unwrap_or_default().to_vec();
        let resp: RESP = res.into_iter().map(|v| v.into()).collect::<Vec<_>>().into();

        write!(self.io, "{resp}")
    }

    pub fn xread(&mut self, mut args: Vec<RESP>) -> std::io::Result<()> {
        let method = args.remove(0).string().unwrap().to_lowercase();

        let time_out: u128 = if method == "block" {
            let r = args
                .remove(0)
                .string()
                .unwrap()
                .parse()
                .unwrap();
            let _streams = args.remove(0).string().unwrap();
            r
        } else {
            1
        };
        
        let now = std::time::Instant::now();

        let stream_count = args.len() / 2;
        let keys: Vec<RESP> = args.drain(0..stream_count).collect();
        let mut starts = vec![];
        
        for arg in args {
            let start = arg.string().unwrap();
            let start = if start == "-" {
                StreamEntryID {time: 0, sqn: 0}
            } else {
                StreamEntryID::implicit(start)
            };
            starts.push(start)
        }
        
        while now.elapsed().as_millis() < time_out {
            let mut store = self.store.lock().unwrap();

            let mut result: Vec<RESP> = vec![];
            for (key, start) in keys.iter().zip(starts.iter()) {
                let stream = store
                    .kv
                    .entry(key.clone().hashable())
                    .or_insert(Value::new_stream())
                    .stream_mut()
                    .unwrap();
                let start = stream.partition_point(|x| &x.id <= start);
                if stream[start..].len() == 0 {
                    continue;
                }
                let resp: RESP = stream
                    .get(start..)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|v| v.clone().into())
                    .collect::<Vec<_>>()
                    .into();
                result.push(vec![key.clone(), resp].into())
            }
            
            if result.len() == 0 {
                drop(store);
                sleep(Duration::from_millis(1));
                continue;
            }

            let resp: RESP = result.into();
            return write!(self.io, "{resp}")
        }

        let resp = RESP::null_array();
        write!(self.io, "{resp}")
    }
}
