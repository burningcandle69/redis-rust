use crate::redis::errors::wrong_num_arguments;
use crate::resp::RESP;
use super::Redis;
use super::redis::Command;

impl Redis {
    pub fn multi(&mut self, _: Command) -> std::io::Result<()> {
        let resp: RESP = "OK".into();
        self.is_transaction = true;
        write!(self.io, "{resp}")
    }
    
    pub fn transaction(&mut self, cmd: Command) -> std::io::Result<()> {
        if cmd[0].clone().string().ok_or(wrong_num_arguments("exec"))?.to_lowercase() != "exec" {
            self.transaction.push(cmd);
            let resp: RESP = "QUEUED".into();
            write!(self.io, "{resp}")
        } else {
            self.is_transaction = false;
            let commands: Vec<_> = self.transaction.drain( ..).collect();
            
            #[cfg(debug_assertions)]
            println!("running queued commands: \n{:?}", commands);
            
            for v in commands {
                self.execute(v)?;
            }
            Ok(())
        }
    }
}