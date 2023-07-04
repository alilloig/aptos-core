// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use aptos_state_view::TStateView;
use aptos_types::{block_executor::partitioner::BlockExecutorTransactions, executable::Executable};
use crate::task::{ExecutorTask, Transaction};

pub trait BlockExecutor {
    type Transaction: Transaction;
    type ExecutorTask: ExecutorTask<Txn = Self::Transaction>;
    type StateView: TStateView<Key = <Self::Transaction as Transaction>::Key> + Sync;
    type Executable: Executable + 'static;
    type Error;

    fn execute_block(
        &self,
        executor_arguments: <Self::ExecutorTask as ExecutorTask>::Argument,
        signature_verified_block: BlockExecutorTransactions<Self::Transaction>,
        base_view: &Self::StateView,
    ) -> Result<Vec<<Self::ExecutorTask as ExecutorTask>::Output>, Self::Error>;
}
