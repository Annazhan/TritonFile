use std::str::FromStr;
use std::io::{BufRead, BufReader, ErrorKind};
use actix_files::Files;
use actix_web::{web, App, HttpServer};
use clap::Parser;
use front::client_fs::lab::{new_bin_client, serve_back};
use front::client_fs::{client::new_client, front::Front};
use log::{info, warn, LevelFilter, error};

use tribbler::config::Config;
use tribbler::config::DEFAULT_CONFIG_LOCATION;
use tribbler::error::{TritonFileResult, TritonFileError};
// use tribbler::ref_impl::RefServer;
use tribbler::disfuser_server::DisfuserServer;
use fuser::{
    Filesystem, KernelConfig, MountOption, ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory,
    ReplyEmpty, ReplyEntry, ReplyOpen, ReplyStatfs, ReplyWrite, ReplyXattr, Request, TimeOrNow,
    FUSE_ROOT_ID,
};

#[derive(Debug, Clone)]
enum ServerType {
    Ref,
    Lab,
}

impl FromStr for ServerType {
    type Err = TritonFileError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ref" => Ok(ServerType::Ref),
            "lab" => Ok(ServerType::Lab),
            _ => Err(TritonFileError::Unknown(format!(
                "{} not a valid ServerType",
                s
            ))),
        }
    }
}

/// A program which runs the tribbler front-end service.
#[derive(Parser, Debug)]
#[clap(name = "trib-front")]
struct Cfg {
    /// level to use when logging
    #[clap(short, long, default_value = "INFO")]
    log_level: LevelFilter,

    /// server type to run the front-end against
    #[clap(short, long, default_value = "ref")]
    server_type: ServerType,

    #[clap(short, long, default_value = DEFAULT_CONFIG_LOCATION)]
    config: String,

    /// the host address to bind to. e.g. 127.0.0.1 or 0.0.0.0
    #[clap(long, default_value = "0.0.0.0")]
    host: String,

    /// the host port to bind
    #[clap(long, default_value = "8080")]
    port: u16,
}

fn main() -> TritonFileResult<()> {
    let args = Cfg::parse();

    env_logger::builder()
        .default_format()
        .filter_level(args.log_level)
        .init();

    let cfg = Config::read(Some(&args.config))?;
    let runtime = tokio::runtime::Builder::new_current_thread()
    .enable_io()
    .build()
    .unwrap();    
    let bin_client_pre = new_bin_client(cfg.backs);
    let bin_client = runtime.block_on(bin_client_pre).unwrap();

    let mut options = vec![MountOption::FSName(format!("fuser"))];
    #[cfg(feature = "abi-7-26")]
    {
        options.push(MountOption::AutoUnmount);
    }
    #[cfg(not(feature = "abi-7-26"))]
    {
        options.push(MountOption::AutoUnmount);
        options.push(MountOption::AllowOther);
        options.push(MountOption::Dev);
        // options.push(MountOption::AllowRoot);
    }
    // let handle = Handle::current();
    let front =  Front::new(
        bin_client,
        runtime
    );

    info!("The file system is ready to be mounted");
    let result = fuser::mount2(
        front, 
        "/Users/lynnz/Desktop/tmp",
        &options,
    );
    dbg!(&result);
    if let Err(e) = result {
                // Return a special error code for permission denied, which usually indicates that
                // "user_allow_other" is missing from /etc/fuse.conf
        if e.kind() == ErrorKind::PermissionDenied {
            error!("{}", e.to_string());
            std::process::exit(2);
        }
    }
    Ok(())

    // let server: web::Data<Srv> = web::Data::new(srv_impl);
    // match populate(&server).await {
    //     Ok(_) => info!("Pre-populated test-server successfully"),
    //     Err(e) => warn!("Failed to pre-populate test server: {}", e),
    // }
    // let srv = HttpServer::new(move || {
    //     App::new()
    //         .app_data(server.clone())
    //         .service(
    //             web::scope("/api")
    //                 .service(api::add_user)
    //                 .service(api::list_users)
    //                 .service(api::list_tribs)
    //                 .service(api::list_home)
    //                 .service(api::is_following)
    //                 .service(api::follow)
    //                 .service(api::unfollow)
    //                 .service(api::following)
    //                 .service(api::post),
    //         )
    //         .service(Files::new("/", "./www").index_file("index.html"))
    // })
    // .bind((args.host.as_str(), args.port))?
    // .run();
    // info!("============================================");
    // info!(
    //     "TRIBBLER SERVING AT ::: http://{}:{}",
    //     &args.host, &args.port
    // );
    // info!("============================================");
    // srv.await?;
    // Ok(())
}
