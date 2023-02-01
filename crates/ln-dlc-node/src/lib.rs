mod disk;
mod ln;
mod setup;

pub use setup::start_ln_dlc_node;

#[cfg(test)]
mod tests;

use dlc_manager::custom_signer::CustomKeysManager;
use dlc_manager::custom_signer::CustomSigner;
use dlc_messages::message_handler::MessageHandler as DlcMessageHandler;
use dlc_sled_storage_provider::SledStorageProvider;
use electrs_blockchain_provider::ElectrsBlockchainProvider;
use lightning::chain::chainmonitor;
use lightning::chain::Filter;
use lightning::chain::{self};
use lightning::ln::peer_handler::IgnoringMessageHandler;
use lightning::ln::PaymentHash;
use lightning::ln::PaymentPreimage;
use lightning::ln::PaymentSecret;
use lightning::routing::gossip::P2PGossipSync;
use lightning::routing::gossip::{self};
use lightning::util::logger::Logger;
use lightning::util::logger::Record;
use lightning_net_tokio::SocketDescriptor;
use lightning_persister::FilesystemPersister;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

type ChainMonitor = chainmonitor::ChainMonitor<
    CustomSigner,
    Arc<dyn Filter + Send + Sync>,
    Arc<ElectrsBlockchainProvider>,
    Arc<ElectrsBlockchainProvider>,
    Arc<TracingLogger>,
    Arc<FilesystemPersister>,
>;

type PeerManager = lightning::ln::peer_handler::PeerManager<
    SocketDescriptor,
    Arc<ChannelManager>,
    Arc<
        P2PGossipSync<
            Arc<lightning::routing::gossip::NetworkGraph<Arc<TracingLogger>>>,
            Arc<dyn chain::Access + Send + Sync>,
            Arc<TracingLogger>,
        >,
    >,
    Arc<IgnoringMessageHandler>,
    Arc<TracingLogger>,
    Arc<DlcMessageHandler>,
>;

type ChannelManager = lightning::ln::channelmanager::ChannelManager<
    Arc<ChainMonitor>,
    Arc<ElectrsBlockchainProvider>,
    Arc<CustomKeysManager>,
    Arc<ElectrsBlockchainProvider>,
    Arc<TracingLogger>,
>;

type SimpleWallet =
    simple_wallet::SimpleWallet<Arc<ElectrsBlockchainProvider>, Arc<SledStorageProvider>>;

type NetworkGraph = gossip::NetworkGraph<Arc<TracingLogger>>;

type PaymentInfoStorage = Arc<Mutex<HashMap<PaymentHash, PaymentInfo>>>;

struct PaymentInfo {
    preimage: Option<PaymentPreimage>,
    secret: Option<PaymentSecret>,
    status: HTLCStatus,
    amt_msat: MillisatAmount,
}

enum HTLCStatus {
    Pending,
    Succeeded,
    Failed,
}

#[derive(Debug)]
struct MillisatAmount(Option<u64>);

#[derive(Copy, Clone)]
struct TracingLogger;

impl Logger for TracingLogger {
    fn log(&self, record: &Record) {
        match record.level {
            lightning::util::logger::Level::Gossip => {
                tracing::trace!(msg = record.args.as_str())
            }
            lightning::util::logger::Level::Trace => {
                tracing::trace!(msg = record.args.as_str())
            }
            lightning::util::logger::Level::Debug => {
                tracing::debug!(msg = record.args.as_str())
            }
            lightning::util::logger::Level::Info => {
                tracing::info!(msg = record.args.as_str())
            }
            lightning::util::logger::Level::Warn => {
                tracing::warn!(msg = record.args.as_str())
            }
            lightning::util::logger::Level::Error => {
                tracing::error!(msg = record.args.as_str())
            }
        };
    }
}
