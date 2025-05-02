use crate::versedb_capnp::versedb;
use capnp::Error;
use capnp::capability::Promise;
use capnp_rpc::{RpcSystem, rpc_twoparty_capnp, twoparty};
use futures::AsyncReadExt;
use futures::AsyncWriteExt;
use std::fmt;
use std::net::ToSocketAddrs;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt as _, BufReader};
use tokio_util::compat::TokioAsyncReadCompatExt;

#[derive(Debug)]
pub enum ClientError {
    CapnpError(Error),
    Utf8Error(std::str::Utf8Error),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::CapnpError(e) => write!(f, "CapnP error: {}", e),
            ClientError::Utf8Error(e) => write!(f, "UTF-8 error: {}", e),
        }
    }
}

impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ClientError::CapnpError(e) => Some(e),
            ClientError::Utf8Error(e) => Some(e),
        }
    }
}

impl From<Error> for ClientError {
    fn from(err: Error) -> Self {
        ClientError::CapnpError(err)
    }
}

impl From<std::str::Utf8Error> for ClientError {
    fn from(err: std::str::Utf8Error) -> Self {
        ClientError::Utf8Error(err)
    }
}

pub struct VerseDbClient {
    client: versedb::Client,
}

pub async fn connect(addr: &str) -> Result<VerseDbClient, Box<dyn std::error::Error>> {
    let addr = addr
        .to_socket_addrs()?
        .next()
        .expect("could not parse address");

    let stream = tokio::net::TcpStream::connect(&addr).await?;
    stream.set_nodelay(true)?;
    let (reader, writer) = TokioAsyncReadCompatExt::compat(stream).split();
    let rpc_network = Box::new(twoparty::VatNetwork::new(
        futures::io::BufReader::new(reader),
        futures::io::BufWriter::new(writer),
        rpc_twoparty_capnp::Side::Client,
        Default::default(),
    ));
    let mut rpc_system = RpcSystem::new(rpc_network, None);
    let client: versedb::Client = rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);

    tokio::task::spawn_local(rpc_system);

    Ok(VerseDbClient::new(client))
}

impl VerseDbClient {
    pub fn new(client: versedb::Client) -> Self {
        Self { client }
    }

    pub async fn add(&self, key: &[u8], value: &[u8]) -> Result<(), ClientError> {
        let mut request = self.client.add_request();
        {
            let mut params = request.get();
            params.set_key(key);
            params.set_value(value);
        }
        request.send().promise.await?;
        Ok(())
    }

    pub async fn select(&self, key: &[u8]) -> Result<Vec<u8>, ClientError> {
        let mut request = self.client.select_request();
        {
            let mut params = request.get();
            params.set_key(key);
        }
        let response = request.send().promise.await?;
        Ok(response.get()?.get_value()?.to_vec())
    }

    pub async fn remove(&self, key: &[u8]) -> Result<(), ClientError> {
        let mut request = self.client.remove_request();
        {
            let mut params = request.get();
            params.set_key(key);
        }
        request.send().promise.await?;
        Ok(())
    }

    pub async fn select_range(
        &self,
        start: &[u8],
        end: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, ClientError> {
        let mut request = self.client.select_range_request();
        {
            let mut params = request.get();
            let mut range = params.init_range();
            range.set_start(start);
            range.set_end(end);
        }
        let response = request.send().promise.await?;
        let pairs = response.get()?.get_pairs()?;
        let mut result = Vec::new();
        for i in 0..pairs.len() {
            let pair = pairs.get(i);
            result.push((pair.get_key()?.to_vec(), pair.get_value()?.to_vec()));
        }
        Ok(result)
    }

    pub async fn remove_range(
        &self,
        start: &[u8],
        end: &[u8],
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, ClientError> {
        let mut request = self.client.remove_range_request();
        {
            let mut params = request.get();
            let mut range = params.init_range();
            range.set_start(start);
            range.set_end(end);
        }
        let response = request.send().promise.await?;
        let pairs = response.get()?.get_pairs()?;
        let mut result = Vec::new();
        for i in 0..pairs.len() {
            let pair = pairs.get(i);
            result.push((pair.get_key()?.to_vec(), pair.get_value()?.to_vec()));
        }
        Ok(result)
    }

    pub async fn helloworld(&self, input: &str) -> Result<String, ClientError> {
        let mut request = self.client.helloworld_request();
        {
            let mut params = request.get();
            params.set_input(input);
        }
        let response = request.send().promise.await?;
        Ok(response.get()?.get_output()?.to_str()?.to_string())
    }

    pub async fn flush(&self) -> Result<(), ClientError> {
        let request = self.client.flush_request();
        request.send().promise.await?;
        Ok(())
    }
}

async fn print_menu() {
    println!("\nVerseDB Interactive Client");
    println!("-------------------------");
    println!("1. Add key-value pair");
    println!("2. Remove key");
    println!("3. Select value by key");
    println!("4. Select range");
    println!("5. Remove range");
    println!("6. Hello world");
    println!("7. Flush");
    println!("0. Exit");
    print!("\nEnter your choice: ");
    io::stdout().flush().await.unwrap();
}

async fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().await.unwrap();
    let mut input = String::new();
    let mut reader = BufReader::new(io::stdin());
    reader.read_line(&mut input).await.unwrap();
    input.trim().to_string()
}

#[tokio::main]
#[cfg(not(target_arch = "wasm32"))]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let local = tokio::task::LocalSet::new();

    local
        .run_until(async move {
            let client = connect("127.0.0.1:8000").await?;

            loop {
                print_menu().await;
                let choice = get_input("").await;

                match choice.as_str() {
                    "1" => {
                        let key = get_input("Enter key: ").await;
                        let value = get_input("Enter value: ").await;
                        client.add(key.as_bytes(), value.as_bytes()).await?;
                        println!("Key-value pair added successfully!");
                    }
                    "2" => {
                        let key = get_input("Enter key to remove: ").await;
                        client.remove(key.as_bytes()).await?;
                        println!("Key removed successfully!");
                    }
                    "3" => {
                        let key = get_input("Enter key to select: ").await;
                        let result = client.select(key.as_bytes()).await?;
                        println!("Value: {}", String::from_utf8_lossy(&result));
                    }
                    "4" => {
                        let start_key = get_input("Enter start key: ").await;
                        let end_key = get_input("Enter end key: ").await;
                        let range_result = client
                            .select_range(start_key.as_bytes(), end_key.as_bytes())
                            .await?;
                        println!("\nRange results:");
                        for (k, v) in range_result {
                            println!(
                                "Key: {}, Value: {}",
                                String::from_utf8_lossy(&k),
                                String::from_utf8_lossy(&v)
                            );
                        }
                    }
                    "5" => {
                        let start_key = get_input("Enter start key: ").await;
                        let end_key = get_input("Enter end key: ").await;
                        let removed = client
                            .remove_range(start_key.as_bytes(), end_key.as_bytes())
                            .await?;
                        println!("\nRemoved range results:");
                        for (k, v) in removed {
                            println!(
                                "Key: {}, Value: {}",
                                String::from_utf8_lossy(&k),
                                String::from_utf8_lossy(&v)
                            );
                        }
                        println!("Range removed successfully!");
                    }
                    "6" => {
                        let name = get_input("Enter name: ").await;
                        let result = client.helloworld(&name).await?;
                        println!("{}", result);
                    }
                    "7" => {
                        client.flush().await?;
                        println!("Database flushed successfully!");
                    }
                    "0" => {
                        println!("Goodbye!");
                        break;
                    }
                    _ => println!("Invalid choice! Please try again."),
                }
            }

            Ok(())
        })
        .await
}
