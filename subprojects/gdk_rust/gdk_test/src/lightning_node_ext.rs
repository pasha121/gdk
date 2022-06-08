use crate::RpcNodeExt;
use bitcoincore_rpc::Client as BitcoinClient;
use clightningrpc::{responses, LightningRPC};
use gdk_greenlight::GreenlightSession;
use gl_client::pb;
use std::{str::FromStr, time::Duration};

pub trait LightningNodeExt {
    fn node_id(&self) -> String;
    fn wait_channel(&self, destination: &str);
    fn fund(&self, arg: u64, client: &BitcoinClient);
    fn wait_for(&self, txid: String);
    fn connect_to_me(&self, g1: &mut GreenlightSession);
}

impl LightningNodeExt for LightningRPC {
    fn wait_channel(&self, destination: &str) {
        let source = self.node_id();
        for _i in 0..60 {
            let list = self.listchannels(None).unwrap();
            for el in list.channels {
                if el.source == source && el.destination == destination {
                    return;
                }
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        assert!(false, "cannot see channel")
    }

    fn fund(&self, arg: u64, client: &BitcoinClient) {
        let address_resp = self.newaddr(None).unwrap();
        dbg!(&address_resp);
        let rec_addr = bitcoin::Address::from_str(&address_resp.bech32.unwrap()).unwrap();

        let txid = RpcNodeExt::sendtoaddress(client, &rec_addr.to_string(), arg, None).unwrap();

        RpcNodeExt::generate(client, 1, None).unwrap();

        self.wait_for(txid);
    }

    fn wait_for(&self, txid: String) {
        for _ in 0..60 {
            let resp = self.listfunds().unwrap();
            let received = resp.outputs.iter().any(|a| a.txid == txid);
            if received {
                return;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        assert!(false, "waited 30 seconds without seeing the tx");
    }

    fn connect_to_me(&self, gl: &mut GreenlightSession) {
        let l1_info = self.getinfo().unwrap();
        let l1_node_id = l1_info.id;

        for net_addr in l1_info.binding.iter() {
            if let responses::NetworkAddress::Ipv4 {
                address,
                port,
            } = net_addr
            {
                gl.connect_peer(pb::ConnectRequest {
                    node_id: l1_node_id.clone(),
                    addr: format!("{}:{}", address, port),
                })
                .unwrap();
            }
        }
    }

    fn node_id(&self) -> String {
        self.getinfo().unwrap().id
    }
}
