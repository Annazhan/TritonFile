#![allow(clippy::needless_return)]

use clap::{crate_version, Arg, Command};
use front::client_fs::lab::new_bin_client;
use fuser::consts::FOPEN_DIRECT_IO;
#[cfg(feature = "abi-7-26")]
use fuser::consts::FUSE_HANDLE_KILLPRIV;
#[cfg(feature = "abi-7-31")]
use fuser::consts::FUSE_WRITE_KILL_PRIV;
use fuser::TimeOrNow::Now;
use fuser::{
    Filesystem, KernelConfig, MountOption, ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory,
    ReplyEmpty, ReplyEntry, ReplyOpen, ReplyStatfs, ReplyWrite, ReplyXattr, Request, TimeOrNow,
    FUSE_ROOT_ID,
};
#[cfg(feature = "abi-7-26")]
use log::info;
use log::{debug, warn};
use log::{error, LevelFilter};
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;
use std::cmp::min;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, ErrorKind, Read, Seek, SeekFrom, Write};
use std::os::raw::c_int;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::FileExt;
#[cfg(target_os = "linux")]
use std::os::unix::io::IntoRawFd;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{env, fs, io};

pub mod client_fs;
use crate::client_fs::front::Front;

fn fuse_allow_other_enabled() -> io::Result<bool> {
    let file = File::open("/etc/fuse.conf")?;
    for line in BufReader::new(file).lines() {
        if line?.trim_start().starts_with("user_allow_other") {
            return Ok(true);
        }
    }
    Ok(false)
}

fn main() {
    let mut back_addr: Vec<String> = Vec::new(); 
    back_addr.push("127.0.0.1:32309".to_string());
    back_addr.push("127.0.0.1:32310".to_string());
    back_addr.push("127.0.0.1:32311".to_string());

    let runtime = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap();    
    let bin_client_pre = new_bin_client(back_addr);
    let bin_client = runtime.block_on(bin_client_pre).unwrap();
    
    let matches = Command::new("Fuser")
    .version(crate_version!())
    .author("Christopher Berner")
    .arg(
        Arg::new("data-dir")
            .long("data-dir")
            .value_name("DIR")
            .default_value("/tmp/fuser")
            .help("Set local directory used to store data")
            .takes_value(true),
    )
    .arg(
        Arg::new("mount-point")
            .long("mount-point")
            .value_name("MOUNT_POINT")
            .default_value("")
            .help("Act as a client, and mount FUSE at given path")
            .takes_value(true),
    )
    .arg(
        Arg::new("direct-io")
            .long("direct-io")
            .requires("mount-point")
            .help("Mount FUSE with direct IO"),
    )
    .arg(Arg::new("fsck").long("fsck").help("Run a filesystem check"))
    .arg(
        Arg::new("suid")
            .long("suid")
            .help("Enable setuid support when run as root"),
    )
    .arg(
        Arg::new("v")
            .short('v')
            .multiple_occurrences(true)
            .help("Sets the level of verbosity"),
    )
    .get_matches();

let verbosity: u64 = matches.occurrences_of("v");
let log_level = match verbosity {
    0 => LevelFilter::Error,
    1 => LevelFilter::Warn,
    2 => LevelFilter::Info,
    3 => LevelFilter::Debug,
    _ => LevelFilter::Trace,
};
env_logger::builder()
    .format_timestamp_nanos()
    .filter_level(log_level)
    .init();

let mut options = vec![MountOption::FSName("fuser".to_string())];

#[cfg(feature = "abi-7-26")]
{
    if matches.is_present("suid") {
        info!("setuid bit support enabled");
        options.push(MountOption::Suid);
    } else {
        options.push(MountOption::AutoUnmount);
    }
}
#[cfg(not(feature = "abi-7-26"))]
{
    options.push(MountOption::AutoUnmount);
}
if let Ok(enabled) = fuse_allow_other_enabled() {
    if enabled {
        options.push(MountOption::AllowOther);
    }
} else {
    eprintln!("Unable to read /etc/fuse.conf");
}

let data_dir: String = matches.value_of("data-dir").unwrap_or_default().to_string();

let mountpoint: String = matches
    .value_of("mount-point")
    .unwrap_or_default()
    .to_string();

let result = fuser::mount2(
    Front::new(
        bin_client, 
        runtime
    ),
    mountpoint,
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
}
