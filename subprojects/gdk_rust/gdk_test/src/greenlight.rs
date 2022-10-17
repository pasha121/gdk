use std::{env, fs};

use bip39::Mnemonic;
use bitcoin::Network;
use bitcoincore_rpc::{Auth, Client as BitcoinClient};
use clightningrpc::LightningRPC;
use gdk_common::greenlight::LoginInfo;
use gdk_common::NetworkParameters;
use gdk_greenlight::GreenlightSession;
use tempfile::TempDir;

pub fn network_parameters(with_certs: bool) -> (NetworkParameters, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let mut network_parameters = NetworkParameters::default();
    network_parameters.development = true;

    if with_certs {
        network_parameters.greenlight_url = env::var("GL_SCHEDULER_GRPC_URI").unwrap();
        network_parameters.nobody_crt = load_file_env("GL_NOBODY_CRT");
        network_parameters.nobody_key = load_file_env("GL_NOBODY_KEY");
        network_parameters.ca_crt = load_file_env("GL_CA_CRT");
    }

    network_parameters.state_dir = format!("{}", temp_dir.path().display());
    (network_parameters, temp_dir)
}

pub fn load_file_env(arg: &str) -> String {
    fs::read_to_string(env::var(arg).unwrap()).unwrap()
}

pub fn register(network_parameters: &NetworkParameters) -> (Mnemonic, LoginInfo) {
    assert_eq!(Network::Regtest, network_parameters.network());

    let mnemonic = Mnemonic::generate_in(bip39::Language::English, 12).unwrap();
    let login_info = GreenlightSession::register(&network_parameters, &mnemonic);
    assert!(login_info.is_ok(), "cannot register");
    (mnemonic, login_info.unwrap())
}

pub fn init() -> (NetworkParameters, TempDir, Mnemonic, LoginInfo) {
    let _ = env_logger::try_init();
    let (network_parameters, temp_dir) = network_parameters(true);
    let (mnemonic, login_info) = register(&network_parameters);
    (network_parameters, temp_dir, mnemonic, login_info)
}

pub fn bitcoin_client() -> BitcoinClient {
    let port = std::env::var("GL_BITCOIND_RPCPORT").unwrap();
    let url = format!("localhost:{}", port);
    BitcoinClient::new(&url, Auth::UserPass("rpcuser".to_string(), "rpcpass".to_string())).unwrap()
}

pub fn lightning_client() -> LightningRPC {
    let port = std::env::var("GL_L1_PORT").unwrap();
    let sock = format!("/tmp/unix-sock-{}", port);
    LightningRPC::new(sock)
}
