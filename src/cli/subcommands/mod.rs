// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

// Due to https://git.wiki.kernel.org/index.php/GitFaq#Why_does_Git_not_.22track.22_renames.3F
// we cannot rewire the git history of this file.
// check out the original commit history here:
// https://github.com/ChainSafe/forest/commits/main/forest/src/cli/mod.rs

mod archive_cmd;
mod attach_cmd;
mod auth_cmd;
mod car_cmd;
mod chain_cmd;
mod db_cmd;
mod fetch_params_cmd;
mod info_cmd;
mod mpool_cmd;
mod net_cmd;
pub mod send_cmd;
mod shutdown_cmd;
mod snapshot_cmd;
mod state_cmd;
mod sync_cmd;
mod wallet_cmd;

use std::io::{self, Write};

use crate::blocks::tipset_json::TipsetJson;
pub(crate) use crate::cli_shared::cli::Config;
use crate::cli_shared::cli::{CliOpts, HELP_MESSAGE};
use crate::utils::version::FOREST_VERSION_STRING;
use cid::Cid;
use clap::Parser;
use jsonrpc_v2::Error as JsonRpcError;
use serde::Serialize;
use tracing::error;

pub(super) use self::{
    archive_cmd::ArchiveCommands, attach_cmd::AttachCommand, auth_cmd::AuthCommands,
    car_cmd::CarCommands, chain_cmd::ChainCommands, db_cmd::DBCommands,
    fetch_params_cmd::FetchCommands, mpool_cmd::MpoolCommands, net_cmd::NetCommands,
    send_cmd::SendCommand, shutdown_cmd::ShutdownCommand, snapshot_cmd::SnapshotCommands,
    state_cmd::StateCommands, sync_cmd::SyncCommands, wallet_cmd::WalletCommands,
};
use crate::cli::subcommands::info_cmd::InfoCommand;

/// CLI structure generated when interacting with Forest binary
#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"), author = env!("CARGO_PKG_AUTHORS"), version = FOREST_VERSION_STRING.as_str(), about = env!("CARGO_PKG_DESCRIPTION"))]
#[command(help_template(HELP_MESSAGE))]
pub struct Cli {
    #[command(flatten)]
    pub opts: CliOpts,
    #[command(subcommand)]
    pub cmd: Subcommand,
}

// This subcommand is hidden and only here to help users migrating to forest-tool
#[derive(clap::Subcommand, Debug)]
pub enum ConfigCommands {
    Dump,
}

/// Forest binary sub-commands available.
#[derive(clap::Subcommand, Debug)]
pub enum Subcommand {
    /// Download parameters for generating and verifying proofs for given size
    #[command(name = "fetch-params")]
    Fetch(FetchCommands),

    /// Interact with Filecoin blockchain
    #[command(subcommand)]
    Chain(ChainCommands),

    /// Manage RPC permissions
    #[command(subcommand)]
    Auth(AuthCommands),

    /// Manage P2P network
    #[command(subcommand)]
    Net(NetCommands),

    /// Manage wallet
    #[command(subcommand)]
    Wallet(WalletCommands),

    /// Inspect or interact with the chain synchronizer
    #[command(subcommand)]
    Sync(SyncCommands),

    /// Interact with the message pool
    #[command(subcommand)]
    Mpool(MpoolCommands),

    /// Interact with and query Filecoin chain state
    #[command(subcommand)]
    State(StateCommands),

    // This subcommand is hidden and only here to help users migrating to forest-tool
    #[command(hide = true)]
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Manage snapshots
    #[command(subcommand)]
    Snapshot(SnapshotCommands),

    /// Manage archives
    #[command(subcommand)]
    Archive(ArchiveCommands),

    /// Send funds between accounts
    Send(SendCommand),

    /// Print node info
    #[command(subcommand)]
    Info(InfoCommand),

    /// Database management
    #[command(subcommand)]
    DB(DBCommands),

    /// Attach to daemon via a JavaScript console
    Attach(AttachCommand),

    /// Shutdown Forest
    Shutdown(ShutdownCommand),

    /// Utilities for manipulating CAR files
    #[command(subcommand)]
    Car(CarCommands),
}

/// Pretty-print a JSON-RPC error and exit
pub(super) fn handle_rpc_err(e: JsonRpcError) -> anyhow::Error {
    match serde_json::to_string(&e) {
        Ok(err_msg) => anyhow::Error::msg(err_msg),
        Err(err) => err.into(),
    }
}

/// Format a vector to a prettified string
pub(super) fn format_vec_pretty(vec: Vec<String>) -> String {
    format!("[{}]", vec.join(", "))
}

/// Print an error message and exit the program with an error code
/// Used for handling high level errors such as invalid parameters
pub fn cli_error_and_die(msg: impl AsRef<str>, code: i32) -> ! {
    error!("Error: {}", msg.as_ref());
    std::process::exit(code);
}

/// Prints a plain HTTP JSON-RPC response result
pub(super) fn print_rpc_res(res: Result<String, JsonRpcError>) -> anyhow::Result<()> {
    let obj = res.map_err(handle_rpc_err)?;
    println!("{}", &obj);
    Ok(())
}

/// Prints a pretty HTTP JSON-RPC response result
pub(super) fn print_rpc_res_pretty<T: Serialize>(
    res: Result<T, JsonRpcError>,
) -> anyhow::Result<()> {
    let obj = res.map_err(handle_rpc_err)?;
    println!("{}", serde_json::to_string_pretty(&obj)?);
    Ok(())
}

/// Prints a tipset from a HTTP JSON-RPC response result
pub(super) fn print_rpc_res_cids(res: Result<TipsetJson, JsonRpcError>) -> anyhow::Result<()> {
    let tipset = res.map_err(handle_rpc_err)?;
    println!(
        "{}",
        serde_json::to_string_pretty(
            &tipset
                .0
                .cids()
                .iter()
                .map(|cid: &Cid| cid.to_string())
                .collect::<Vec<_>>()
        )?
    );
    Ok(())
}

/// Prints a bytes HTTP JSON-RPC response result
pub(super) fn print_rpc_res_bytes(res: Result<Vec<u8>, JsonRpcError>) -> anyhow::Result<()> {
    let obj = res.map_err(handle_rpc_err)?;
    println!(
        "{}",
        String::from_utf8(obj).map_err(|e| handle_rpc_err(e.into()))?
    );
    Ok(())
}

/// Prints a string HTTP JSON-RPC response result to a buffered `stdout`
pub(super) fn print_stdout(out: String) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle
        .write_all(out.as_bytes())
        .map_err(|e| handle_rpc_err(e.into()))
        .unwrap();

    handle
        .write("\n".as_bytes())
        .map_err(|e| handle_rpc_err(e.into()))
        .unwrap();
}

pub fn prompt_confirm() -> bool {
    print!("Do you want to continue? [y/n] ");
    std::io::stdout().flush().unwrap();
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    let line = line.trim().to_lowercase();
    line == "y" || line == "yes"
}
