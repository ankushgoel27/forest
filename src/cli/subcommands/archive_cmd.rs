// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

//! Archives are key-value pairs encoded as
//! [CAR files](https://ipld.io/specs/transport/car/carv1/). The key-value pairs
//! represent a directed, acyclic graph (DAG). This graph is often a subset of a larger
//! graph and references to missing keys are common.
//!
//! Each graph contains blocks, messages, state trees, and miscellaneous data
//! such as compiled `WASM` code. The amount of data differs greatly in different
//! kinds of archives. While there are no fixed definitions, there are three
//! common kind of archives:
//! - A full archive contains a complete graph with no missing nodes. These
//!   archives are large (14 TiB for Filecoin's mainnet) and only used in special
//!   situations.
//! - A lite-archive typically has roughly 3 million blocks, 2000 complete sets of
//!   state-roots, and 2000 sets of messages. These archives usually take up
//!   roughly 100 GiB.
//! - A diff-archive contains the subset of nodes that are _not_ shared by two
//!   other archives. These archives are much smaller but can rarely be used on
//!   their own. They are typically merged with other archives before use.
//!
//! The sub-commands in this module manipulate archive files without needing a
//! running Forest-daemon or a separate database. Operations are carried out
//! directly on CAR files.
//!
//! Additional reading: [`crate::db::car::plain`]

use crate::blocks::Tipset;
use crate::chain::{
    index::{ChainIndex, ResolveNullTipset},
    ChainEpochDelta,
};
use crate::cli_shared::{snapshot, snapshot::TrustedVendor};
use crate::db::car::ManyCar;
use crate::ipld::{stream_graph, CidHashSet};
use crate::networks::{calibnet, mainnet, ChainConfig, NetworkChain};
use crate::shim::clock::{ChainEpoch, EPOCHS_IN_DAY, EPOCH_DURATION_SECONDS};
use anyhow::{bail, Context as _};
use chrono::NaiveDateTime;
use clap::Subcommand;
use futures::TryStreamExt;
use fvm_ipld_blockstore::Blockstore;
use sha2::Sha256;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Subcommand)]
pub enum ArchiveCommands {
    // This subcommand is hidden and only here to help users migrating to forest-tool
    #[command(hide = true)]
    Info { snapshot: PathBuf },
    /// Trim a snapshot of the chain and write it to `<output_path>`
    Export {
        /// Snapshot input path. Currently supports only `.car` file format.
        #[arg(required = true)]
        snapshot_files: Vec<PathBuf>,
        /// Snapshot output filename or directory. Defaults to
        /// `./forest_snapshot_{chain}_{year}-{month}-{day}_height_{epoch}.car.zst`.
        #[arg(short, long, default_value = ".", verbatim_doc_comment)]
        output_path: PathBuf,
        /// Latest epoch that has to be exported for this snapshot, the upper bound. This value
        /// cannot be greater than the latest epoch available in the input snapshot.
        #[arg(short, long)]
        epoch: Option<ChainEpoch>,
        /// How many state-roots to include. Lower limit is 900 for `calibnet` and `mainnet`.
        #[arg(short, long, default_value_t = 2000)]
        depth: ChainEpochDelta,
        /// Do not include any values reachable from epoch-diff.
        #[arg(short, long)]
        diff: Option<ChainEpochDelta>,
    },
    /// Print block headers at 30 day interval for a snapshot file
    Checkpoints {
        /// Path to snapshot file.
        #[arg(required = true)]
        snapshot_files: Vec<PathBuf>,
    },
}

impl ArchiveCommands {
    pub async fn run(self) -> anyhow::Result<()> {
        match self {
            Self::Export {
                snapshot_files,
                output_path,
                epoch,
                depth,
                diff,
            } => {
                let store = ManyCar::try_from(snapshot_files)?;
                let heaviest_tipset = store.heaviest_tipset()?;
                do_export(store, heaviest_tipset, output_path, epoch, depth, diff).await
            }
            Self::Checkpoints {
                snapshot_files: snapshot,
            } => print_checkpoints(snapshot),
            Self::Info { .. } => crate::bail_moved_cmd!("archive info"),
        }
    }
}

// This does nothing if the output path is a file. If it is a directory - it produces the following:
// `./forest_snapshot_{chain}_{year}-{month}-{day}_height_{epoch}.car.zst`.
fn build_output_path(
    chain: String,
    genesis_timestamp: u64,
    epoch: ChainEpoch,
    output_path: PathBuf,
) -> PathBuf {
    match output_path.is_dir() {
        true => output_path.join(snapshot::filename(
            TrustedVendor::Forest,
            chain,
            NaiveDateTime::from_timestamp_opt(
                genesis_timestamp as i64 + epoch * EPOCH_DURATION_SECONDS,
                0,
            )
            .unwrap_or_default()
            .into(),
            epoch,
            true,
        )),
        false => output_path.clone(),
    }
}

async fn do_export(
    store: impl Blockstore + Send + Sync + 'static,
    root: Tipset,
    output_path: PathBuf,
    epoch_option: Option<ChainEpoch>,
    depth: ChainEpochDelta,
    diff: Option<ChainEpochDelta>,
) -> anyhow::Result<()> {
    let ts = Arc::new(root);

    let genesis = ts.genesis(&store)?;
    let network = if genesis.cid() == &*calibnet::GENESIS_CID {
        NetworkChain::Calibnet
    } else if genesis.cid() == &*mainnet::GENESIS_CID {
        NetworkChain::Mainnet
    } else {
        NetworkChain::Devnet("devnet".to_string())
    };

    let epoch = epoch_option.unwrap_or(ts.epoch());

    let finality = ChainConfig::from_chain(&network)
        .policy
        .chain_finality
        .min(epoch);
    if depth < finality {
        bail!("For {}, depth has to be at least {}.", network, finality);
    }

    info!("looking up a tipset by epoch: {}", epoch);

    let index = ChainIndex::new(&store);

    let ts = index
        .tipset_by_height(epoch, ts, ResolveNullTipset::TakeOlder)
        .context("unable to get a tipset at given height")?;

    let seen = if let Some(diff) = diff {
        let diff_ts: Arc<Tipset> = index
            .tipset_by_height(diff, ts.clone(), ResolveNullTipset::TakeOlder)
            .context("diff epoch must be smaller than target epoch")?;
        let diff_ts: &Tipset = &diff_ts;
        let mut stream = stream_graph(&store, diff_ts.clone().chain(&store));
        while stream.try_next().await?.is_some() {}
        stream.into_seen()
    } else {
        CidHashSet::default()
    };

    let output_path =
        build_output_path(network.to_string(), genesis.timestamp(), epoch, output_path);

    let writer = tokio::fs::File::create(&output_path)
        .await
        .context(format!(
            "unable to create a snapshot - is the output path '{}' correct?",
            output_path.to_str().unwrap_or_default()
        ))?;

    info!(
        "exporting snapshot at location: {}",
        output_path.to_str().unwrap_or_default()
    );

    let pb = indicatif::ProgressBar::new_spinner().with_style(
        indicatif::ProgressStyle::with_template(
            "{spinner} exported {total_bytes} with {binary_bytes_per_sec} in {elapsed}",
        )
        .expect("indicatif template must be valid"),
    );
    pb.enable_steady_tick(std::time::Duration::from_secs_f32(0.1));
    let writer = pb.wrap_async_write(writer);

    crate::chain::export::<Sha256>(store, &ts, depth, writer, seen, true).await?;

    Ok(())
}

// Print a mapping of epochs to block headers in yaml format. This mapping can
// be used by Forest to quickly identify tipsets.
fn print_checkpoints(snapshot_files: Vec<PathBuf>) -> anyhow::Result<()> {
    let store = ManyCar::try_from(snapshot_files).context("couldn't read input CAR file")?;
    let root = store.heaviest_tipset()?;

    let genesis = root.genesis(&store)?;
    let chain_name = if genesis.cid() == &*calibnet::GENESIS_CID {
        NetworkChain::Calibnet
    } else if genesis.cid() == &*mainnet::GENESIS_CID {
        NetworkChain::Mainnet
    } else {
        bail!("Unrecognizable genesis block");
    };

    println!("{}:", chain_name);
    for (epoch, cid) in list_checkpoints(store, root) {
        println!("  {}: {}", epoch, cid);
    }
    Ok(())
}

fn list_checkpoints(
    db: impl Blockstore,
    root: Tipset,
) -> impl Iterator<Item = (ChainEpoch, cid::Cid)> {
    let interval = EPOCHS_IN_DAY * 30;
    let mut target_epoch = root.epoch() - root.epoch() % interval;
    root.chain(db).filter_map(move |tipset| {
        if tipset.epoch() <= target_epoch && tipset.epoch() != 0 {
            target_epoch -= interval;
            Some((tipset.epoch(), *tipset.min_ticket_block().cid()))
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::car::AnyCar;
    use async_compression::tokio::bufread::ZstdDecoder;
    use fvm_ipld_car::CarReader;
    use tempfile::TempDir;
    use tokio::io::BufReader;
    use tokio_util::compat::TokioAsyncReadCompatExt;

    fn genesis_timestamp(genesis_car: &'static [u8]) -> u64 {
        let db = crate::db::car::PlainCar::try_from(genesis_car).unwrap();
        let ts = db.heaviest_tipset().unwrap();
        ts.genesis(&db).unwrap().timestamp()
    }

    #[tokio::test]
    async fn export() {
        let output_path = TempDir::new().unwrap();
        let store = AnyCar::try_from(calibnet::DEFAULT_GENESIS).unwrap();
        let heaviest_tipset = store.heaviest_tipset().unwrap();
        do_export(
            store,
            heaviest_tipset,
            output_path.path().into(),
            Some(0),
            1,
            None,
        )
        .await
        .unwrap();
        let file = tokio::fs::File::open(build_output_path(
            NetworkChain::Calibnet.to_string(),
            genesis_timestamp(calibnet::DEFAULT_GENESIS),
            0,
            output_path.path().into(),
        ))
        .await
        .unwrap();
        let file = BufReader::new(file);
        CarReader::new(ZstdDecoder::new(file).compat())
            .await
            .unwrap();
    }
}
