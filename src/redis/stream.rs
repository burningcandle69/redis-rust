use crate::redis::Redis;
use crate::redis::utils::make_io_error;
use crate::redis::value::{StreamEntry, StreamEntryID, Value};
use crate::resp::RESP;
use std::collections::HashMap;

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
        let mut store = self.store.lock().unwrap();
        assert!(
            args.remove(0).string().unwrap().to_lowercase() == "streams",
            "streams keyword should be there"
        );

        let stream_count = args.len() / 2;
        let keys: Vec<RESP> = args.drain(0..stream_count).collect();

        let mut result: Vec<RESP> = vec![];
        for key in keys {
            let stream = store
                .kv
                .entry(key.clone().hashable())
                .or_insert(Value::new_stream())
                .stream_mut()
                .unwrap();
            let start = args.remove(0).string().unwrap();
            let start = if start == "-" {
                0
            } else {
                let id = StreamEntryID::implicit(start);
                stream.partition_point(|x| x.id <= id)
            };
            let resp: RESP = stream
                .get(start..)
                .unwrap_or_default()
                .into_iter()
                .map(|v| v.clone().into())
                .collect::<Vec<_>>()
                .into();
            result.push(vec![key, resp].into())
        }

        let resp: RESP = result.into();
        write!(self.io, "{resp}")
    }
}
