use bitcoin::Network;
use bitcoincore_rpc::RpcApi;
use clightningrpc::requests;
use gdk_greenlight::How;
use gdk_greenlight::{AuthenticateInput, GreenlightSession, SetLocalEncryptionKeysInput};
use gdk_test::greenlight::{bitcoin_client, init, lightning_client};
use gdk_test::{GreenlightSessionExt, LightningNodeExt, RpcNodeExt};
use gl_client::pb;
use serde_json::{json, Value};

#[ignore]
#[test]
fn test_bitcoin_client() {
    let client = bitcoin_client();
    assert!(client.get_network_info().is_ok());
}

#[ignore]
#[test]
fn test_lightning_client() {
    let client = lightning_client();
    let getinfo = client.getinfo();
    assert!(getinfo.is_ok());
}

#[ignore]
#[test]
fn test_recover() {
    let (network_parameters, _temp_dir, mnemonic, login_info) = init();

    let login_info_recovered = GreenlightSession::recover(&network_parameters, &mnemonic).unwrap();
    assert_eq!(login_info_recovered.node_id, login_info.node_id);

    // recover non existing
    let mnemonic = bip39::Mnemonic::generate_in(bip39::Language::English, 12).unwrap();
    let login_info = GreenlightSession::recover(&network_parameters, &mnemonic);
    assert!(login_info.is_err());
}

#[ignore]
#[test]
fn test_register() {
    let (network_parameters, _temp_dir, mnemonic, _login_info) = init();

    let login_info = GreenlightSession::register(&network_parameters, &mnemonic);
    assert!(login_info.is_err(), "already registered doesn't fail");
}

#[ignore]
#[test]
fn test_get_info() {
    let (network_parameters, _temp_dir, mnemonic, login_info) = init();

    let mut session = GreenlightSession::new(network_parameters).unwrap();
    session.login(login_info, mnemonic).unwrap();
    assert!(session.get_info().is_ok());
}

#[ignore]
#[test]
fn test_connect_testnet() {
    let _ = env_logger::try_init();
    let (mut network_parameters, _temp_dir) = gdk_test::greenlight::network_parameters(true);
    network_parameters.development = false;
    assert_eq!(Network::Testnet, network_parameters.network());

    let mnemonic = bip39::Mnemonic::parse("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about").unwrap();
    let login_info = GreenlightSession::recover(&network_parameters, &mnemonic).unwrap();

    let mut session = GreenlightSession::new(network_parameters).unwrap();
    session.login(login_info, mnemonic).unwrap();
    assert!(session.get_info().is_ok());

    let res = session.connect_peer(pb::ConnectRequest {
        node_id: "02661ec31ab6e97aaec73bbfe1845e88a333b8650db4e1be13a82af9b0c7413a11".to_string(), // Blockstream testnet node
        addr: "35.185.91.205:19735".to_string(),
    });
    assert!(res.is_ok());
}

#[ignore]
#[test]
fn test_create_invoice() {
    let (network_parameters, _temp_dir, mnemonic, login_info) = init();

    let mut session = GreenlightSession::new(network_parameters).unwrap();
    session.login(login_info, mnemonic).unwrap();

    let invoice = GreenlightSessionExt::create_invoice(&mut session, 1000);

    assert!(invoice.starts_with("lnbcrt10u"));
}

#[ignore]
#[test]
fn test_authenticate() {
    let (network_parameters, _temp_dir, mnemonic, _login_info) = init();
    let bitcoin_network = network_parameters.network();

    let mut session = GreenlightSession::new(network_parameters).unwrap();

    let xpub =
        GreenlightSession::mnemonic_to_xpub_for_encrypting(&mnemonic, bitcoin_network).unwrap();
    session.set_local_encryption_keys(SetLocalEncryptionKeysInput {
        xpub,
    });

    let output = session
        .authenticate(AuthenticateInput {
            mnemonic: mnemonic.to_string(),
        })
        .unwrap();
    assert_eq!(output.how, How::Recover);

    let output = session
        .authenticate(AuthenticateInput {
            mnemonic: mnemonic.to_string(),
        })
        .unwrap();
    assert_eq!(output.how, How::Disk);

    assert!(session.get_info().is_ok());
}

#[ignore]
#[test]
fn test_payment_to_gl() {
    let (network_parameters, _temp_dir, mnemonic, login_info) = init();
    let bitcoin_client = bitcoin_client();
    let lightning_client = lightning_client();

    let mut s1 = GreenlightSession::create_and_login(&network_parameters, login_info, mnemonic);

    lightning_client.connect_to_me(&mut s1);

    lightning_client.fund(1_000_000, &bitcoin_client);

    let _result = lightning_client
        .fundchannel(&s1.node_id().unwrap(), requests::AmountOrAll::Amount(500_000), None)
        .unwrap();

    RpcNodeExt::generate(&bitcoin_client, 1, None).unwrap();

    lightning_client.wait_channel(&s1.node_id().unwrap());

    let invoice = GreenlightSessionExt::create_invoice(&mut s1, 50_000);

    let res = lightning_client.call::<Value, Value>("pay", json!([&invoice]));

    assert!(res.is_ok());
}

#[ignore]
#[test]
fn test_payment_from_gl() {
    let (network_parameters, _temp_dir, mnemonic, login_info) = init();
    let bitcoin_client = bitcoin_client();
    let lightning_client = lightning_client();
    let mut s1 = GreenlightSession::create_and_login(&network_parameters, login_info, mnemonic);

    s1.fund(1_000_000, &bitcoin_client);
    lightning_client.connect_to_me(&mut s1);

    // TODO finish the test, at the moment is failing for feerate

    // let res = lightning_client.call::<Value, Value>("dev-feerate", json!([s1_node_id, 1])); // it looks it's not working

    let _result = s1.fund_channel_sat(&lightning_client.node_id(), 500_000);
    // assert!(result.is_ok()) // fee rate error, FIXME

    // s1.fund_channel(req).unwrap();
}
