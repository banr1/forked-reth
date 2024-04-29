//! Traits for configuring a node

use crate::{primitives::NodePrimitives, ConfigureEvm, EngineTypes};
use reth_db::{
    database::Database,
    database_metrics::{DatabaseMetadata, DatabaseMetrics},
};
use reth_network::NetworkHandle;
use reth_payload_builder::PayloadBuilderHandle;
use reth_provider::FullProvider;
use reth_tasks::TaskExecutor;
use reth_transaction_pool::TransactionPool;
use std::marker::PhantomData;

/// The type that configures the essential types of an ethereum like node.
///
/// This includes the primitive types of a node, the engine API types for communication with the
/// consensus layer, and the EVM configuration type for setting up the Ethereum Virtual Machine.
pub trait NodeTypes: Send + Sync + 'static {
    /// The node's primitive types, defining basic operations and structures.
    type Primitives: NodePrimitives;
    /// The node's engine types, defining the interaction with the consensus engine.
    type Engine: EngineTypes;
    /// The node's EVM configuration, defining settings for the Ethereum Virtual Machine.
    type Evm: ConfigureEvm;

    /// Returns the node's evm config.
    fn evm_config(&self) -> Self::Evm;
}

/// A helper trait that is downstream of the [NodeTypes] trait and adds stateful components to the
/// node.
pub trait FullNodeTypes: NodeTypes + 'static {
    /// Underlying database type used by the node to store and retrieve data.
    type DB: Database + DatabaseMetrics + DatabaseMetadata + Clone + Unpin + 'static;
    /// The provider type used to interact with the node.
    type Provider: FullProvider<Self::DB>;
}

/// An adapter type that adds the builtin provider type to the user configured node types.
#[derive(Debug)]
pub struct FullNodeTypesAdapter<Types, DB, Provider> {
    /// An instance of the user configured node types.
    pub types: Types,
    /// The database type used by the node.
    pub db: PhantomData<DB>,
    /// The provider type used by the node.
    pub provider: PhantomData<Provider>,
}

impl<Types, DB, Provider> FullNodeTypesAdapter<Types, DB, Provider> {
    /// Create a new adapter from the given node types.
    pub fn new(types: Types) -> Self {
        Self { types, db: Default::default(), provider: Default::default() }
    }
}

impl<Types, DB, Provider> NodeTypes for FullNodeTypesAdapter<Types, DB, Provider>
where
    Types: NodeTypes,
    DB: Send + Sync + 'static,
    Provider: Send + Sync + 'static,
{
    type Primitives = Types::Primitives;
    type Engine = Types::Engine;
    type Evm = Types::Evm;

    fn evm_config(&self) -> Self::Evm {
        self.types.evm_config()
    }
}

impl<Types, DB, Provider> FullNodeTypes for FullNodeTypesAdapter<Types, DB, Provider>
where
    Types: NodeTypes,
    Provider: FullProvider<DB>,
    DB: Database + DatabaseMetrics + DatabaseMetadata + Clone + Unpin + 'static,
{
    type DB = DB;
    type Provider = Provider;
}

/// Encapsulates all types and components of the node.
pub trait FullNodeComponents: FullNodeTypes + 'static {
    /// The transaction pool of the node.
    type Pool: TransactionPool;

    /// Returns the transaction pool of the node.
    fn pool(&self) -> &Self::Pool;

    /// Returns the provider of the node.
    fn provider(&self) -> &Self::Provider;

    /// Returns the handle to the network
    fn network(&self) -> &NetworkHandle;

    /// Returns the handle to the payload builder service.
    fn payload_builder(&self) -> &PayloadBuilderHandle<Self::Engine>;

    /// Returns the task executor.
    fn task_executor(&self) -> &TaskExecutor;
}