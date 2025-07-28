use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use serde_json::Value;
use std::io::ErrorKind;
use std::str::FromStr;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, split};
use tokio::net::TcpListener;

pub async fn run(addr: String) -> std::io::Result<()> {
    println!("Listening on address {}", addr);
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_connection(&mut socket).await {
                eprintln!("Failed to handle connection: {}", e)
            }
        });
    }
}

async fn send_malformed<T>(stream: &mut T) -> std::io::Result<()>
where
    T: AsyncWrite + Unpin,
{
    let _ = stream.write("Malformed Request".as_bytes()).await?;
    Ok(())
}

// This is generic to support unit testing
async fn handle_connection<T>(stream: &mut T) -> std::io::Result<()>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let (read_stream, mut write_stream) = split(stream);
    let mut reader = BufReader::new(read_stream);
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(n) => {
                if n == 0 {
                    // connection closed
                    break;
                }
                println!("received: {}", line);
                let v: Value = match serde_json::from_str(&line) {
                    Ok(val) => val,
                    Err(_) => {
                        send_malformed(&mut write_stream).await?;
                        return Err(std::io::Error::new(
                            ErrorKind::InvalidData,
                            "Error reading JSON",
                        ));
                    }
                };
                if v["method"] != "isPrime" {
                    println!("Missing isPrime method");
                    send_malformed(&mut write_stream).await?;
                    return Err(std::io::Error::new(
                        ErrorKind::InvalidData,
                        "Missing isPrime method",
                    ));
                }

                let num_str = v["number"].to_string();
                let number = match BigDecimal::from_str(&num_str) {
                    Ok(val) => val,
                    Err(_) => {
                        send_malformed(&mut write_stream).await?;
                        return Err(std::io::Error::new(
                            ErrorKind::InvalidData,
                            "Invalid number",
                        ));
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
                let _ = write_stream.write(ret_value.as_bytes()).await?;
            }
            Err(err) => panic!("{}", err),
        }
    }

    Ok(())
}

pub fn is_prime(n: BigInt) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, duplex};

    #[test]
    fn test_prime() {
        assert!(is_prime(BigInt::from(5)));
    }

    #[test]
    fn test_not_prime() {
        assert!(!is_prime(BigInt::from(6)));
    }

    #[tokio::test]
    async fn test_handle_connection() {
        let (mut client, mut server) = duplex(64);

        // spawn server
        let server_task = tokio::spawn(async move {
            handle_connection(&mut server).await.unwrap();
        });

        client
            .write_all(b"{\"method\":\"isPrime\",\"number\":123}")
            .await
            .unwrap();
        client.shutdown().await.unwrap();

        let mut response: Vec<u8> = vec![];
        client.read_buf(&mut response).await.unwrap();
        assert_eq!(&response, b"{\"method\":\"isPrime\",\"prime\":false}\n");

        server_task.await.unwrap();
    }

    #[tokio::test]
    #[should_panic]
    async fn test_handle_malformed() {
        let (mut client, mut server) = duplex(64);

        // spawn server
        let server_task = tokio::spawn(async move {
            handle_connection(&mut server).await.unwrap();
        });

        client
            .write_all(b"{\"method\":\"is\",\"number\":123}")
            .await
            .unwrap();
        client.shutdown().await.unwrap();

        let mut response: Vec<u8> = vec![];
        client.read_buf(&mut response).await.unwrap();
        assert_eq!(&response, b"Malformed Request");

        server_task.await.unwrap();
    }
}
