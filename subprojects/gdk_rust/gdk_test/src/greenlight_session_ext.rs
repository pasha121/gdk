use std::time::Duration;

use crate::{Error, RpcNodeExt};
use bip39::Mnemonic;
use bitcoin::{
    hashes::hex::{FromHex, ToHex},
    secp256k1::rand::{self, Rng},
};
use bitcoincore_rpc::Client as BitcoinClient;
use gdk_common::{greenlight::LoginInfo, NetworkParameters};
use gdk_greenlight::GreenlightSession;
use gl_client::pb::{self, FundChannelResponse};

pub trait GreenlightSessionExt {
    fn create_and_login(
        network_parameters: &NetworkParameters,
        login_info: LoginInfo,
        mnemonic: Mnemonic,
    ) -> GreenlightSession;
    fn fund(&mut self, arg: u64, client: &BitcoinClient);
    fn wait_for(&mut self, txid: String);
    fn create_invoice(&mut self, amount: u64) -> String;
    fn fund_channel_sat(&mut self, dst: &str, satoshi: u64) -> Result<FundChannelResponse, Error>;
}

impl GreenlightSessionExt for GreenlightSession {
    fn create_and_login(
        network_parameters: &NetworkParameters,
        login_info: LoginInfo,
        mnemonic: Mnemonic,
    ) -> GreenlightSession {
        let mut s1 = GreenlightSession::new(network_parameters.clone()).unwrap();
        s1.login(login_info, mnemonic).unwrap();
        s1
    }

    fn fund(&mut self, arg: u64, client: &BitcoinClient) {
        let address_resp = self.new_addr().unwrap();
        let txid = RpcNodeExt::sendtoaddress(client, &address_resp.address, arg, None).unwrap();
        RpcNodeExt::generate(client, 1, None).unwrap();

        self.wait_for(txid);
    }

    fn wait_for(&mut self, txid: String) {
        for _ in 0..60 {
            let resp = self.list_funds().unwrap();
            let received =
                resp.outputs.iter().flat_map(|e| &e.output).any(|a| a.txid.to_hex() == txid);
            if received {
                return;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        assert!(false, "waited 30 seconds without seeing the tx");
    }

    fn create_invoice(&mut self, amount: u64) -> String {
        let mut rng = rand::thread_rng();

        let invoice: pb::Invoice = self
            .create_invoice(pb::InvoiceRequest {
                amount: Some(pb::Amount {
                    unit: Some(pb::amount::Unit::Satoshi(amount)),
                }),
                description: format!("{}", rng.gen::<u64>()),
                label: format!("{}", rng.gen::<u64>()),
                // preimage: vec![],
            })
            .unwrap();
        invoice.bolt11
    }

    fn fund_channel_sat(&mut self, dst: &str, satoshi: u64) -> Result<FundChannelResponse, Error> {
        let req = pb::FundChannelRequest {
            node_id: Vec::<u8>::from_hex(&dst).unwrap(),
            amount: Some(pb::Amount {
                unit: Some(pb::amount::Unit::Satoshi(satoshi)),
            }),
            feerate: Some(pb::Feerate {
                value: Some(pb::feerate::Value::Perkw(10_000)),
            }),
            announce: true,
            minconf: Some(pb::Confirmation {
                blocks: 1,
            }),
            close_to: "".to_string(),
        };
        Ok(self.fund_channel(req)?)
    }
}
