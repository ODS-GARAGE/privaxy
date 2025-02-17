use std::sync::{Arc, RwLock};
use crossbeam_channel::{Receiver, Sender};
use log::{error};
use reqwest::redirect::Policy;
use crate::proxy::exclusion::LocalExclusionStore;
use crate::blocker::{AdblockRequester, BlockerRequest};
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use crate::events::Event;

pub mod configuration;
mod ca;
mod proxy;
mod blocker;
mod statistics;
mod events;
mod cert;
mod blocker_utils;

#[derive(Debug)]
pub struct PrivaxyServer {
    pub certificate_pem: String,
    pub configuration_updater_sender: mpsc::Sender<configuration::Configuration>,
    pub configuration_save_lock: Arc<tokio::sync::Mutex<()>>,
    pub blocking_disabled_store: blocker::BlockingDisabledStore,
    pub statistics: statistics::Statistics,
    pub local_exclusion_store: LocalExclusionStore,
    pub requests_broadcast_sender: broadcast::Sender<Event>
}

pub async fn start_privaxy() -> PrivaxyServer {
    // let ip = [127, 0, 0, 1]; // localhost address

    // 대부분의 프록시를 수행하기 위해 hyper Client 대신 request 사용함.
    // 압축을 처리하고 더 편리한 인터페이스를 제공하기 떄문.
    let client = reqwest::Client::builder()
        .use_rustls_tls() // tls 사용
        .redirect(Policy::none()) // redirection 따르지 않음.
        .no_proxy() // 사전 지정한 호스트들에 대해 proxy 사용하지 않도록 지정.
        // LocalExclusionStore 에서 exclusion 호스트 list 설정.
        .gzip(true) // 서버에서 gzip 으로 압축된 파일 처리 허용
        .brotli(true) // 서버에서 brotli 로 압축된 파일 처리 허용
        .deflate(true) // 서버에서 deflate 로 압축된 파일 처리 허용
        .build()    // 위 설정 기반으로 Client 빌드
        .unwrap();

    // configuration 모듈의 read_from_home(client: Client) 메서드 호출해 서버의 환경 설정.
    let configuration = match configuration::Configuration::read_from_home(client.clone()).await {
        Ok(configuration) => configuration,
        Err(err) => {
            error!("An error occurred while trying to process the configuration file : {:?}", err);
            std::process::exit(1)
        }
    };

    let local_exclusion_store_clone = LocalExclusionStore::new(configuration.get_exclusion().unwrap()).clone();

    let (ca_certificate, ca_private_key) = configuration.get_certificate().unwrap();
    let certificate_pem = std::str::from_utf8(&ca_certificate.to_pem().unwrap())
        .unwrap()
        .to_string();

    let cert_cache = cert::CertCache::new(ca_certificate, ca_private_key);

    let statistics = statistics::Statistics::new();
    let statistics_clone = statistics.clone();

    let (broadcast_tx, _broadcast_rx) = broadcast::channel(32);
    let broadcast_tx_clone = broadcast_tx.clone();

    let blocking_disabled_store = blocker::BlockingDisabledStore(Arc::new(RwLock::new(false)));
    let blocking_disabled_store_clone = blocking_disabled_store.clone();

    let (crossbeam_sender, crossbeam_receiver):
        (Sender<BlockerRequest>, Receiver<BlockerRequest>) = crossbeam_channel::unbounded();
    let blocker_sender = crossbeam_sender.clone();
    let blocker_requester = AdblockRequester::new(blocker_sender);

    // let configuration_updater = configuration::ConfigurationUpdater


}
