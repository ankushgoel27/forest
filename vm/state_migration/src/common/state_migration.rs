// Copyright 2019-2023 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use ahash::{HashMap, HashMapExt};
use cid::Cid;
use forest_shim::{clock::ChainEpoch, state_tree::StateTree};
use fvm_ipld_blockstore::Blockstore;

use super::{verifier::MigrationVerifier, Migrator, PostMigrationAction};
use crate::common::migration_job::{MigrationJob, MigrationJobOutput};

/// Handles several cases of migration:
/// - nil migrations, essentially mapping one Actor to another,
/// - migrations where state upgrade is required,
/// - creating new actors that were not present in the prior network version.
pub(crate) struct StateMigration<BS> {
    migrations: HashMap<Cid, Migrator<BS>>,
    /// Verifies correctness of the migration specification.
    verifier: Option<MigrationVerifier<BS>>,
    /// Post migration actions. This may include new actor creation.
    post_migration_actions: Vec<PostMigrationAction<BS>>,
}

impl<BS: Blockstore + Clone + Send + Sync> StateMigration<BS> {
    pub(crate) fn new(
        verifier: Option<MigrationVerifier<BS>>,
        post_migration_actions: Vec<PostMigrationAction<BS>>,
    ) -> Self {
        Self {
            migrations: HashMap::new(),
            verifier,
            post_migration_actions,
        }
    }

    /// Inserts a new migrator into the migration specification.
    pub(crate) fn add_migrator(&mut self, prior_cid: Cid, migrator: Migrator<BS>) {
        self.migrations.insert(prior_cid, migrator);
    }

    pub(crate) fn migrate_state_tree(
        &self,
        store: BS,
        prior_epoch: ChainEpoch,
        actors_in: StateTree<BS>,
        mut actors_out: StateTree<BS>,
    ) -> anyhow::Result<Cid> {
        // Checks if the migration specification is correct
        if let Some(verifier) = &self.verifier {
            verifier.verify_migration(&store, &self.migrations, &actors_in)?;
        }

        // we need at least 3 threads for the migration to work
        let threads = num_cpus::get().max(3);
        let chan_size = threads / 2;

        log::info!("Using {threads} threads for migration and channel size of {chan_size}",);

        let pool = rayon::ThreadPoolBuilder::new()
            .thread_name(|id| format!("state migration thread: {id}"))
            .num_threads(threads)
            .build()?;

        let (state_tx, state_rx) = crossbeam_channel::bounded(chan_size);
        let (job_tx, job_rx) = crossbeam_channel::bounded(chan_size);

        pool.scope(|s| {
            let store_clone = store.clone();

            s.spawn(move |_| {
                actors_in
                    .for_each(|addr, state| {
                        state_tx
                            .send((addr, state.clone()))
                            .expect("failed sending actor state through channel");
                        Ok(())
                    })
                    .expect("Failed iterating over actor state");
            });

            s.spawn(move |scope| {
                while let Ok((address, state)) = state_rx.recv() {
                    let job_tx = job_tx.clone();
                    let store_clone = store_clone.clone();
                    let migrator = self.migrations.get(&state.code).cloned().unwrap_or_else(|| panic!("migration failed with state code: {}", state.code));
                    scope.spawn(move |_| {
                        let job = MigrationJob {
                            address,
                            actor_state: state,
                            actor_migration: migrator,
                        };

                        let job_output = job.run(store_clone, prior_epoch).unwrap_or_else(|e| {
                            panic!(
                                "failed executing job for address: {address}, Reason: {e}"
                            )
                        });

                        job_tx.send(job_output).unwrap_or_else(|_| {
                            panic!("failed sending job output for address: {address}")
                        });
                    });
                }
                drop(job_tx);
            });

            while let Ok(job_output) = job_rx.recv() {
                let MigrationJobOutput {
                    address,
                    actor_state,
                } = job_output;
                actors_out
                    .set_actor(&address, actor_state)
                    .unwrap_or_else(|e| {
                        panic!(
                            "Failed setting new actor state at given address: {address}, Reason: {e}"
                        )
                    });
            }
        });

        // execute post migration actions, e.g., create new actors
        for action in self.post_migration_actions.iter() {
            action(&store, &mut actors_out)?;
        }

        actors_out.flush()
    }
}
