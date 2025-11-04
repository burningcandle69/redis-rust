use crate::redis::errors::wrong_num_arguments;
use crate::resp::RESP;
use super::Redis;
use super::redis::Command;

impl Redis {
    pub fn multi(&mut self, _: Command) -> std::io::Result<RESP> {
        self.is_transaction = true;
        Ok("OK".into())
    }

    pub fn transaction(&mut self, cmd: Command) -> std::io::Result<RESP> {
        match cmd.get(0).ok_or(wrong_num_arguments("exec"))?.to_lowercase().as_str() {
            "exec" => {
                self.is_transaction = false;
                let commands: Vec<_> = self.transaction.drain( ..).collect();

                #[cfg(debug_assertions)]
                println!("running queued commands: \n{:?}", commands);

                let mut res = vec![];
                for v in commands {
                    match self.execute(v) {
                        Ok(r) => res.push(r),
                        Err(e) => res.push(RESP::SimpleError(format!("{e}")))
                    }
                }

                Ok(res.into())
            }
            "discard" => {
                self.transaction.drain(..);
                self.is_transaction = false;
                Ok("OK".into())
            }
            _ => {
                self.transaction.push(cmd);
                Ok("QUEUED".into())
            }
        }
    }
}