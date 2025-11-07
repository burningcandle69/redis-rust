use super::server::Server;
use super::{Args, Result};
use crate::server::errors::wrong_num_arguments;

impl Server {
    pub async fn config(&mut self, mut args: Args) -> Result {
        let store = self.store.lock().await;
        let get = args.pop_front().ok_or(wrong_num_arguments("config"))?;
        assert_eq!(get.to_lowercase(), "get");
        let mut res = vec![];
        for key in args {
            res.push(key.clone());
            let val = match key.to_lowercase().as_str() {
                "dir" => store.info.dir.clone(),
                "dbfilename" => store.info.db_filename.clone(),
                _ => unimplemented!(),
            };
            res.push(val);
        }
        Ok(res.into())
    }

    /// Returns all keys matching pattern
    /// ```
    /// KEYS pattern
    /// ```
    pub async fn keys(&mut self, mut args: Args) -> Result {
        let pattern = args.pop_front().ok_or(wrong_num_arguments("keys"))?;
        let pattern = pattern.replace("*", ".*");
        let mut res = vec![];
        let re = regex::Regex::new(&pattern)?;
        for k in self.store.lock().await.kv.keys() {
            if re.is_match_at(k, 0) {
                res.push(k.clone());
            }
        }
        Ok(res.into())
    }
}
