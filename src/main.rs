use serde_json::Value;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::net::Shutdown;
use std::net::{TcpListener, TcpStream};
use std::thread;

// TODO: Works for single client, need to handle multiple connections
fn main() {
    println!("Listening on port 8080");
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();

    for stream in listener.incoming() {
        println!("Starting connection");
        match stream {
            Ok(mut stream) => {
                thread::spawn(move || {
                    process_request(&mut stream);
                });
            }
            Err(err) => panic!("{}", err),
        }
    }
}

fn is_prime(n: i64) -> bool {
    if n <= 1 {
        return false;
    }

    let mut i = 2;
    while i * i < (n + 1) {
        if n % i == 0 {
            return false;
        }
        i += 1;
    }

    true
}

fn send_malformed(stream: &mut TcpStream) {
    let _ = stream.write("Malformed Request".as_bytes());
    let _ = stream.shutdown(Shutdown::Both);
}

fn process_request(stream: &mut TcpStream) {
    let mut reader = BufReader::new(stream.try_clone().expect("stream clone failed"));
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(n) => {
                if n == 0 {
                    // connection closed
                    break;
                }
                println!("received: {}", line);
                let v: Value = match serde_json::from_str(&line) {
                    Ok(val) => val,
                    Err(_) => {
                        send_malformed(stream);
                        return;
                    }
                };
                if v["method"] != "isPrime" {
                    println!("Missing isPrime method");
                    send_malformed(stream);
                    return;
                }
                let number = v["number"].as_i64();
                if number.is_none() {
                    println!("Invalid number: {:?}", v["number"]);
                    send_malformed(stream);
                    return;
                }
                let ret_value = format!(
                    "{{\"method\":\"isPrime\",\"prime\":{}}}\n",
                    is_prime(number.unwrap())
                );
                let _ = stream.write(ret_value.as_bytes());
            }
            Err(err) => panic!("{}", err),
        }
    }
}
