// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::authority::AuthorityStore;
use async_trait::async_trait;
use sui_storage::execution_cache::{ExecutionCache, ObjectReadResult};

pub struct InMemoryCache {
    objects: DashMap<ObjectID, Vec<Arc<Object>>>,

    // TODO: use concurrent LRU?
    transaction_objects: DashMap<TransactionDigest, Vec<Arc<Object>>>,

    //transactions: DashMap<TransactionDigest, Arc<VerifiedTransaction>>,
    //signed_transactions: DashMap<TransactionDigest, Arc<SignedTransaction>>,
    transaction_effects: DashMap<TransactionEffectsDigest, TransactionEffects>,

    executed_effects_digests: DashMap<TransactionDigest, TransactionEffectsDigest>,

    store: Arc<AuthorityStore>,
}

#[async_trait]
pub trait ExecutionCache {
    async fn notify_read_objects_for_signing(
        &self,
        tx_digest: &TransactionDigest,
        objects: &[InputObjectKind],
        timeout: Duration,
    ) -> SuiResult<Vec<Arc<Object>>> {
        todo!()
    }

    async fn lock_transaction(
        &self,
        signed_transaction: VerifiedSignedTransaction,
        mutable_input_objects: &[ObjectRef],
    ) -> SuiResult {
        todo!()
    }

    async fn notify_read_objects_for_execution(
        &self,
        tx_digest: &TransactionDigest,
        objects: &[ObjectKey],
    ) -> SuiResult<Vec<ObjectReadResult>> {
        todo!()
    }

    fn read_child_object(
        &self,
        tx_digest: &TransactionDigest,
        object: &ObjectID,
        version_bound: SequenceNumber,
    ) -> SuiResult<Arc<Object>> {
        todo!()
    }

    fn prefetch_objects(&self, tx_digest: &TransactionDigest, objects: &[ObjectKey]) {}

    async fn write_transaction_outputs(
        &self,
        inner_temporary_store: InnerTemporaryStore,
        effects: &TransactionEffects,
        transaction: &VerifiedTransaction,
        epoch_id: EpochId,
    ) -> SuiResult {
        todo!()
    }

    async fn notify_read_effects_digest(
        &self,
        tx_digest: &TransactionDigest,
    ) -> SuiResult<TransactionEffectsDigest> {
        todo!()
    }

    async fn read_effects(
        &self,
        tx_digest: &TransactionDigest,
    ) -> SuiResult<Option<TransactionEffects>> {
        todo!()
    }
}
