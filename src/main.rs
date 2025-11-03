mod redis;
mod resp;

use redis::Redis;
use std::net::TcpListener;
use std::thread;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| -> std::io::Result<()> {
                    let mut redis = Redis::new(Box::new(stream));
                    redis.handle()
                });
                println!("accepted new connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}
