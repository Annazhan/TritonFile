
use std::net::ToSocketAddrs;
use tokio::sync::mpsc::Receiver;
use tonic::transport::Server;
use tribbler::err::TribblerError;
use tribble::storage::RemoteFileSystem;
use tribbler::rpc::trib_storage_server::TribStorageServer;
use tribbler::{config::BackConfig, err::TribResult, storage::Storage};

pub struct Config {
    pub backs: Vec<String>,
    pub keepers: Vec<String>,
}
impl Config {
    // generates the back config struct
    pub fn single_back_config(
        &self,
        idx: usize,
        ready: Option<Sender<bool>>,
        shutdown: Option<Receiver<()>>,
    ) -> SignleBackConfig {
        SignleBackConfig {
            num: idx,
            addr: self.backs[idx].to_string(),
            ready,
            shutdown,
        }
    }
}

pub struct SignleBackConfig {
    pub num: usize,
    /// the address `<host>:<port>` combination to serve on
    pub addr: String,
    /// a channel which should a single message when the storage is
    /// ready to serve requests. If no channel is present, then nothing
    /// is required of this field.
    pub ready: Option<Sender<bool>>,
    /// When a message is received on this channel, it should trigger a
    /// graceful shutdown of the server. If no channel is present, then
    /// no graceful shutdown mechanism needs to be implemented.
    pub shutdown: Option<Receiver<()>>,
}
use std::fmt::Debug;
impl Debug for SignleBackConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignleBackConfig")
            .field("addr", &self.addr)
            .field("ready", &self.ready)
            .field("shutdown", &self.shutdown)
            .finish()
    }
}

pub async fn multiple_serve_back(config: Config) {
    let (tx, rdy) = mpsc::channel();

    let mut handles = vec![];
    
    for (i, srv) in &config.backs.iter().enumerate() {
        if addr::check(srv)? {
            handles.push(tokio::spawn(run_srv(
                i,
                config.clone(),
                Some(tx.clone()),
            )));
        }
    }

    if handles.is_empty() {
        warn!("no {}s found for this host", proc_name);
        return Ok(());
    }

    for h in handles {
        match join!(h) {
            (Ok(_),) => (),
            (Err(e),) => {
                warn!("A {} failed to join: {}", proc_name, e);
            }
        };
    }
    Ok(())

}


pub async fn run_srv(idx: usize, config: Arc<Config>, tx: Option<Sender<bool>>) {
    // from bins_run.rs
    let cfg = single_back_config(config.single_back_config(idx, tx, None));
    info!("starting backend on {}", cfg.addr);
    single_serve_back(cfg).await;
}


// a single backend starts
pub async fn single_serve_back(config: SignleBackConfig) -> TribResult<()> {
    // addr is the address the server should listen on, in the form of <host>:<port>
    let addr = config.addr.to_socket_addrs();
    match addr {
        Ok(value) => (),
        Err(e) => match config.ready {
            Some(channel) => {
                // send a false when you encounter any error on starting your service.
                channel.send(false).unwrap();
                return Err(Box::new(TribblerError::Unknown(
                    "Invalid addr space".to_string(),
                )));
            }
            None => (),
        },
    }

    // ready is a channel for notifying the other parts in the program that the server is ready to accept RPC calls from the network (indicated by the server sending the value true) or if the setup failed (indicated by sending false).
    // ready might be None, which means the caller does not care about when the server is ready.
    match config.ready {
        // send a true over the ready channel when the service is ready (when ready is not None),
        Some(channel) => {
            channel.send(true).unwrap();
        }
        None => (),
    }

    let next_addr = config.addr.to_socket_addrs().unwrap().next().unwrap();
    let trib_storage_server = RemoteFileSystem::new(config.num);
    let storage_server = Server::builder().add_service(trib_storage_server);

    // shutdown is another type of channel for receiving a shutdown notification.
    // when a message is received on this channel, the server should shut down.
    pub async fn receive(mut receiver: Receiver<()>) {
        receiver.recv().await;
    }

    match config.shutdown {
        Some(channel) => {
            storage_server
                .serve_with_shutdown(next_addr, receive(channel))
                .await?;
        }
        None => {
            storage_server.serve(next_addr).await?;
        }
    };
    Ok(())
}

/// This function should create a new client which implements the [Storage]
/// trait. It should communicate with the backend that is started in the
/// [serve_back] function.
pub async fn new_client(addr: &str) -> TribResult<Box<dyn Storage>> {
    Ok(Box::new(StorageClient {
        addr: addr.to_string(),
    }))
}
