use crate::database::{Database, Result as DbResult};
use crate::sled::SledDatabase;
use crate::versedb_capnp::versedb;
use capnp::Error;
use capnp::capability::{Client, FromServer, Promise};
use capnp_rpc::{RpcSystem, rpc_twoparty_capnp, twoparty};
use futures::AsyncReadExt;
use std::collections::BTreeMap;
use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct VerseDbServer<T: Database + Clone + Send + Sync + 'static> {
    store: Arc<Mutex<T>>,
}

impl<T: Database + Clone + Send + Sync + 'static> VerseDbServer<T> {
    pub fn new(store: T) -> Self {
        Self {
            store: Arc::new(Mutex::new(store)),
        }
    }
}

impl<T: Database + Clone + Send + Sync + 'static> versedb::Server for VerseDbServer<T> {
    fn add(
        &mut self,
        params: versedb::AddParams,
        mut results: versedb::AddResults,
    ) -> Promise<(), Error> {
        let key = params.get().unwrap().get_key().unwrap().to_vec();
        let value = params.get().unwrap().get_value().unwrap().to_vec();

        let store = self.store.clone();
        Promise::from_future(async move {
            store
                .lock()
                .unwrap()
                .add(&key, &value)
                .await
                .map_err(|e| Error::failed(format!("{}", e)))?;
            Ok(())
        })
    }

    fn select(
        &mut self,
        params: versedb::SelectParams,
        mut results: versedb::SelectResults,
    ) -> Promise<(), Error> {
        let key = params.get().unwrap().get_key().unwrap().to_vec();
        let store = self.store.clone();

        Promise::from_future(async move {
            if let Some(value) = store
                .lock()
                .unwrap()
                .select(&key)
                .await
                .map_err(|e| Error::failed(format!("{}", e)))?
            {
                results.get().set_value(&value);
            }
            Ok(())
        })
    }

    fn remove(
        &mut self,
        params: versedb::RemoveParams,
        _results: versedb::RemoveResults,
    ) -> Promise<(), Error> {
        let key = params.get().unwrap().get_key().unwrap().to_vec();
        let store = self.store.clone();

        Promise::from_future(async move {
            store
                .lock()
                .unwrap()
                .remove(&key)
                .await
                .map_err(|e| Error::failed(format!("{}", e)))?;
            Ok(())
        })
    }

    fn select_range(
        &mut self,
        params: versedb::SelectRangeParams,
        mut results: versedb::SelectRangeResults,
    ) -> Promise<(), Error> {
        let range = params.get().unwrap().get_range().unwrap();
        let start = range.get_start().unwrap().to_vec();
        let end = range.get_end().unwrap().to_vec();
        let store = self.store.clone();

        Promise::from_future(async move {
            let pairs = store
                .lock()
                .unwrap()
                .select_range(&start, &end)
                .await
                .map_err(|e| Error::failed(format!("{}", e)))?;
            let mut pairs_builder = results.get().init_pairs(pairs.len() as u32);

            for (i, (key, value)) in pairs.iter().enumerate() {
                let mut pair = pairs_builder.reborrow().get(i as u32);
                pair.set_key(key);
                pair.set_value(value);
            }

            Ok(())
        })
    }

    fn remove_range(
        &mut self,
        params: versedb::RemoveRangeParams,
        mut results: versedb::RemoveRangeResults,
    ) -> Promise<(), Error> {
        let range = params.get().unwrap().get_range().unwrap();
        let start = range.get_start().unwrap().to_vec();
        let end = range.get_end().unwrap().to_vec();
        let store = self.store.clone();

        Promise::from_future(async move {
            let pairs = store
                .lock()
                .unwrap()
                .remove_range(&start, &end)
                .await
                .map_err(|e| Error::failed(format!("{}", e)))?;
            let mut pairs_builder = results.get().init_pairs(pairs.len() as u32);

            for (i, (key, value)) in pairs.iter().enumerate() {
                let mut pair = pairs_builder.reborrow().get(i as u32);
                pair.set_key(key);
                pair.set_value(value);
            }

            Ok(())
        })
    }

    fn helloworld(
        &mut self,
        params: versedb::HelloworldParams,
        mut results: versedb::HelloworldResults,
    ) -> Promise<(), Error> {
        let input = params.get().unwrap().get_input().unwrap();
        let input_str = input.to_str().unwrap();
        results.get().set_output(&format!("Hello, {}!", input_str));
        Promise::ok(())
    }

    fn flush(
        &mut self,
        _params: versedb::FlushParams,
        _results: versedb::FlushResults,
    ) -> Promise<(), Error> {
        let store = self.store.clone();
        Promise::from_future(async move {
            store
                .lock()
                .unwrap()
                .flush()
                .await
                .map_err(|e| Error::failed(format!("{}", e)))?;
            Ok(())
        })
    }
}

impl<T: Database + Clone + Send + Sync + 'static> versedb::Server for Arc<VerseDbServer<T>> {
    fn add(
        &mut self,
        params: versedb::AddParams,
        results: versedb::AddResults,
    ) -> Promise<(), Error> {
        let mut server = self.as_ref().clone();
        server.add(params, results)
    }

    fn select(
        &mut self,
        params: versedb::SelectParams,
        results: versedb::SelectResults,
    ) -> Promise<(), Error> {
        let mut server = self.as_ref().clone();
        server.select(params, results)
    }

    fn remove(
        &mut self,
        params: versedb::RemoveParams,
        results: versedb::RemoveResults,
    ) -> Promise<(), Error> {
        let mut server = self.as_ref().clone();
        server.remove(params, results)
    }

    fn select_range(
        &mut self,
        params: versedb::SelectRangeParams,
        results: versedb::SelectRangeResults,
    ) -> Promise<(), Error> {
        let mut server = self.as_ref().clone();
        server.select_range(params, results)
    }

    fn remove_range(
        &mut self,
        params: versedb::RemoveRangeParams,
        results: versedb::RemoveRangeResults,
    ) -> Promise<(), Error> {
        let mut server = self.as_ref().clone();
        server.remove_range(params, results)
    }

    fn helloworld(
        &mut self,
        params: versedb::HelloworldParams,
        results: versedb::HelloworldResults,
    ) -> Promise<(), Error> {
        let mut server = self.as_ref().clone();
        server.helloworld(params, results)
    }

    fn flush(
        &mut self,
        params: versedb::FlushParams,
        results: versedb::FlushResults,
    ) -> Promise<(), Error> {
        let mut server = self.as_ref().clone();
        server.flush(params, results)
    }
}

pub async fn run_server<T: Database + Clone + Send + Sync + 'static>(
    addr: &str,
    store: T,
) -> anyhow::Result<()> {
    let addr = addr
        .to_socket_addrs()?
        .next()
        .expect("could not parse address");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Server listening on {}", addr);

    let server = VerseDbServer::new(store);
    let server = Arc::new(server);
    let local = tokio::task::LocalSet::new();

    local
        .run_until(async move {
            let result: anyhow::Result<()> = async move {
                loop {
                    let (stream, _) = listener.accept().await?;
                    stream.set_nodelay(true)?;
                    let stream = tokio_util::compat::TokioAsyncReadCompatExt::compat(stream);
                    let (reader, writer) = stream.split();

                    let rpc_network = Box::new(twoparty::VatNetwork::new(
                        reader,
                        writer,
                        rpc_twoparty_capnp::Side::Server,
                        Default::default(),
                    ));

                    let server = server.clone();
                    let client: versedb::Client = capnp_rpc::new_client(server);
                    let rpc_system = RpcSystem::new(rpc_network, Some(client.client));

                    tokio::task::spawn_local(rpc_system);
                }
            }
            .await;
            result
        })
        .await?;

    Ok(())
}

#[tokio::main]
#[cfg(not(target_arch = "wasm32"))]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a CSV database
    let db = SledDatabase::open("data.sled").await?;

    // Run the server on localhost:8000 with the CSV database
    run_server("127.0.0.1:8000", db).await?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Empty main function for wasm32 target
}
