use crate::client_fs::binstore::BinStore;
use crate::client_fs::front::Front;
use fuser::Filesystem;
use log::info;
use std::net::ToSocketAddrs;
use std::sync::mpsc::Sender;
use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc,
    },
    thread,
    time::Duration,
};
use tokio::net::TcpListener;
use tonic::transport::Server;
use tribbler::error::{TritonFileError, TritonFileResult};
use tribbler::rpc::trib_storage_server::TribStorageServer;
use tribbler::storage::BinStorage;
use tribbler::storage::RemoteFileSystem;
use tribbler::{config::BackConfig, storage::Storage};

fn send_signal(chan: &Option<Sender<bool>>, signal: bool) -> TritonFileResult<()> {
    match chan {
        Some(sender) => match sender.send(signal) {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(e)),
        },
        None => Ok(()),
    }
}

pub async fn serve_back(config: BackConfig) -> TritonFileResult<()> {
    info!("Serv_back request, config: {:?}", &config);
    let addr = match config.addr.to_socket_addrs() {
        Ok(mut a) => a.next().unwrap(),
        Err(e) => {
            send_signal(&config.ready, false)?;
            return Err(Box::new(e));
        }
    };
    info!("Serv_back request, addr: {:?}", &addr);
    let incoming = match TcpListener::bind(addr).await {
        Ok(lis) => tokio_stream::wrappers::TcpListenerStream::new(lis),
        Err(e) => {
            send_signal(&config.ready, false)?;
            info!("to start the server, the error is {:?}", e);
            return Err(Box::new(e));
        }
    };

    let trib_storage_server = tribbler::disfuser_server::DisfuserServer::new(config.storage);
    let server = Server::builder().add_service(
        tribbler::disfuser::disfuser_server::DisfuserServer::new(trib_storage_server),
    );
    // let server = Server::builder().add_service(TribStorageServer::new(server::StorageServer::new(
    //     config.storage,
    // )));

    send_signal(&config.ready, true)?;
    match config.shutdown {
        Some(mut recver) => {
            server
                .serve_with_incoming_shutdown(incoming, async {
                    recver.recv().await;
                })
                .await?
        }
        None => server.serve_with_incoming(incoming).await?,
    };
    Ok(())
}

pub async fn new_bin_client(backs: Vec<String>) -> TritonFileResult<Box<dyn BinStorage>> {
    Ok(Box::new(BinStore::new(backs)))
}

// pub async fn new_front(
//     bin_storage: Box<dyn BinStorage>,
// ) -> TritonFileResult<Box<dyn Filesystem + Send + Sync>> {
//     info!("New_front request");
//     Ok(Box::new(Front::new{
//         bin_storage

//     }))
// }
