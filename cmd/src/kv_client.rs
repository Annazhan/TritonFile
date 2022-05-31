use clap::{Command, Parser};
use cmd::client_cmds::{app_commands, match_storage_cmds, repl};
use front::client_fs::{lab, client::new_client};
use tribbler::error::TritonFileResult;
#[allow(unused_imports)]
use tribbler::storage::{KeyList, KeyString, KeyValue, Pattern};

#[derive(Parser, Debug)]
#[clap(name = "kv-client")]
struct Options {
    #[clap(short, long, default_value = "127.0.0.1:7799")]
    address: String,

    #[clap(short, long)]
    log: bool,
}

#[tokio::main]
async fn main() -> TritonFileResult<()> {
    let options = Options::parse();
    let client = new_client(&format!("http://{}", &options.address)).await?;
    let app = Command::new("kv-client").subcommands(app_commands());

    loop {
        match repl(&app) {
            Ok(subcmd) => match match_storage_cmds(&*client, subcmd.subcommand()).await {
                true => continue,
                false => break,
            },
            Err(_) => continue,
        };
    }
    Ok(())
}
