use bigdecimal::BigDecimal;
use num_bigint::{BigInt, ParseBigIntError};
use serde_json::Value;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::net::Shutdown;
use std::net::{TcpListener, TcpStream};
use std::str::FromStr;
use std::thread;

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

fn is_prime(n: BigInt) -> bool {
    if n <= BigInt::from(1) {
        return false;
    }

    let mut i = BigInt::from(2);
    while &i * &i < (&n + BigInt::from(1)) {
        if &n % &i == BigInt::from(0) {
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

                let num_str = v["number"].to_string();
                let number = match BigDecimal::from_str(&num_str) {
                    Ok(val) => val,
                    Err(_) => {
                        send_malformed(stream);
                        return;
                    }
                };

                let ret_value: String = if number.fractional_digit_count() > 0 {
                    format!("{{\"method\":\"isPrime\",\"prime\":{}}}\n", false)
                } else {
                    let numint = number.into_bigint_and_scale();
                    format!(
                        "{{\"method\":\"isPrime\",\"prime\":{}}}\n",
                        is_prime(numint.0)
                    )
                };
                let _ = stream.write(ret_value.as_bytes());
            }
            Err(err) => panic!("{}", err),
        }
    }
}
