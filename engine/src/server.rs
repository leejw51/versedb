use capnp::capability::{Promise, Client, FromServer};
use capnp::Error;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use crate::versedb_capnp::versedb;
use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use futures::AsyncReadExt;
use std::net::ToSocketAddrs;

#[derive(Clone)]
pub struct VerseDbServer {
    store: Arc<Mutex<BTreeMap<Vec<u8>, Vec<u8>>>>,
}

impl VerseDbServer {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
}

impl versedb::Server for VerseDbServer {
    fn add(
        &mut self,
        params: versedb::AddParams,
        mut results: versedb::AddResults,
    ) -> Promise<(), Error> {
        let key = params.get().unwrap().get_key().unwrap().to_vec();
        let value = params.get().unwrap().get_value().unwrap().to_vec();
        
        self.store.lock().unwrap().insert(key, value);
        Promise::ok(())
    }

    fn select(
        &mut self,
        params: versedb::SelectParams,
        mut results: versedb::SelectResults,
    ) -> Promise<(), Error> {
        let key = params.get().unwrap().get_key().unwrap().to_vec();
        
        if let Some(value) = self.store.lock().unwrap().get(&key) {
            results.get().set_value(value);
        }
        Promise::ok(())
    }

    fn remove(
        &mut self,
        params: versedb::RemoveParams,
        _results: versedb::RemoveResults,
    ) -> Promise<(), Error> {
        let key = params.get().unwrap().get_key().unwrap().to_vec();
        
        self.store.lock().unwrap().remove(&key);
        Promise::ok(())
    }

    fn select_range(
        &mut self,
        params: versedb::SelectRangeParams,
        mut results: versedb::SelectRangeResults,
    ) -> Promise<(), Error> {
        let range = params.get().unwrap().get_range().unwrap();
        let start = range.get_start().unwrap().to_vec();
        let end = range.get_end().unwrap().to_vec();
        
        let store = self.store.lock().unwrap();
        let pairs: Vec<_> = store.range(start..end).map(|(k, v)| (k.clone(), v.clone())).collect();
        let mut pairs_builder = results.get().init_pairs(pairs.len() as u32);
        
        for (i, (key, value)) in pairs.iter().enumerate() {
            let mut pair = pairs_builder.reborrow().get(i as u32);
            pair.set_key(key);
            pair.set_value(value);
        }
        
        Promise::ok(())
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
}

impl versedb::Server for Arc<VerseDbServer> {
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

    fn helloworld(
        &mut self,
        params: versedb::HelloworldParams,
        results: versedb::HelloworldResults,
    ) -> Promise<(), Error> {
        let mut server = self.as_ref().clone();
        server.helloworld(params, results)
    }
}

pub async fn run_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let addr = addr
        .to_socket_addrs()?
        .next()
        .expect("could not parse address");

    tokio::task::LocalSet::new()
        .run_until(async move {
            let listener = tokio::net::TcpListener::bind(&addr).await?;
            let versedb_server = Arc::new(VerseDbServer::new());
            let versedb_client: versedb::Client = capnp_rpc::new_client(versedb_server);

            println!("VerseDB server listening on {}", addr);

            loop {
                let (stream, _) = listener.accept().await?;
                stream.set_nodelay(true)?;
                let (reader, writer) =
                    tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();
                let network = twoparty::VatNetwork::new(
                    futures::io::BufReader::new(reader),
                    futures::io::BufWriter::new(writer),
                    rpc_twoparty_capnp::Side::Server,
                    Default::default(),
                );

                let rpc_system =
                    RpcSystem::new(Box::new(network), Some(versedb_client.clone().client));

                tokio::task::spawn_local(rpc_system);
            }
        })
        .await
}
