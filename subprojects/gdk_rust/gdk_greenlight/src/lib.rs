use aes_gcm_siv::Aes256GcmSiv;
use bip39::Mnemonic;
use bitcoin::hashes::hex::{FromHex, ToHex};
use bitcoin::util::bip32::{ExtendedPrivKey, ExtendedPubKey};
use bitcoin::{secp256k1, Network};
use gdk_common::exchange_rates::{ExchangeRatesCache, ExchangeRatesCacher};
use gdk_common::greenlight::LoginInfo;
use gdk_common::notification::NativeNotif;
use gdk_common::session::{JsonError, NewSession, Session};
use gdk_common::store::{Decryptable, Encryptable, ToCipher};
use gdk_common::NetworkParameters;
use gl_client::pb::*;
use gl_client::{node::Client, scheduler::Scheduler, signer::Signer, tls::TlsConfig};
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, to_value, Value};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::thread::JoinHandle;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc::Sender;

pub use error::Error;

mod error;

pub struct GreenlightSession {
    network_parameters: NetworkParameters,
    runtime: Runtime,
    client: Option<Client>,
    signer_handle: Option<SignerHandle>,
    xr_cache: ExchangeRatesCache,
    notify: NativeNotif,
    node_id_hex: Option<String>,

    /// Used to encrypt local cache
    cipher: Option<Aes256GcmSiv>,
}

pub struct SignerHandle {
    _handle: JoinHandle<Result<(), Error>>,
    shutdown_signal: Sender<()>,
}

impl Drop for GreenlightSession {
    fn drop(&mut self) {
        if let Some(signer_handle) = self.signer_handle.take() {
            let _ = self.runtime.block_on(signer_handle.shutdown_signal.send(()));
            //let _ = signer_handle.handle.join(); // TODO uncomment when it doesn't freeze
        }
    }
}

impl ExchangeRatesCacher for GreenlightSession {
    fn xr_cache(&self) -> &ExchangeRatesCache {
        &self.xr_cache
    }

    fn xr_cache_mut(&mut self) -> &mut ExchangeRatesCache {
        &mut self.xr_cache
    }
}

impl GreenlightSession {
    pub fn new(network_parameters: NetworkParameters) -> Result<Self, Error> {
        let runtime = Builder::new_current_thread().enable_io().enable_time().build()?;

        Ok(GreenlightSession {
            network_parameters,
            runtime,
            client: None,
            signer_handle: None,
            notify: NativeNotif::new(),
            cipher: None,
            node_id_hex: None,
            xr_cache: ExchangeRatesCache::new(),
        })
    }
    pub fn node_id(&self) -> Option<&str> {
        self.node_id_hex.as_deref()
    }
    pub fn mnemonic_to_xpub_for_encrypting(
        mnemonic: &Mnemonic,
        network: Network,
    ) -> Result<ExtendedPubKey, Error> {
        let seed = mnemonic.to_seed("");
        let secp = secp256k1::Secp256k1::new(); // TODO move EC to gdk_common and use that
        let master_xprv = ExtendedPrivKey::new_master(network, &seed)?;
        let master_xpub = ExtendedPubKey::from_priv(&secp, &master_xprv);
        // TODO is it ok to use the master xpub?
        Ok(master_xpub)
    }
}

impl NewSession for GreenlightSession {
    fn new_session(network_parameters: NetworkParameters) -> Result<Box<dyn Session>, JsonError> {
        Ok(Box::new(GreenlightSession::new(network_parameters)?))
    }
}

impl Session for GreenlightSession {
    fn handle_call(&mut self, method: &str, input: Value) -> Result<Value, JsonError> {
        debug!("handle_call {}", method);

        if method.starts_with("internal_") {
            return Ok(self.handle_internal_call(&method[9..], input)?);
        }

        match method {
            "connect_peer" => Ok(to_value(self.connect_peer(from_value(input)?)?)?),
            "disconnect" => Ok(to_value(self.disconnect(from_value(input)?)?)?),
            "fund_channel" => Ok(to_value(self.fund_channel(from_value(input)?)?)?),
            "get_info" => Ok(to_value(self.get_info()?)?),
            "create_invoice" => Ok(to_value(self.create_invoice(from_value(input)?)?)?),
            "keysend" => Ok(to_value(self.keysend(from_value(input)?)?)?),
            "list_funds" => Ok(to_value(self.list_funds()?)?),
            "list_invoice" => Ok(to_value(self.list_invoice()?)?),
            "list_payments" => Ok(to_value(self.list_payments()?)?),
            "list_peers" => Ok(to_value(self.list_peers()?)?),
            "new_addr" => Ok(to_value(self.new_addr()?)?),
            "pay" => Ok(to_value(self.pay(from_value(input)?)?)?),
            "withdraw" => Ok(to_value(self.withdraw(from_value(input)?)?)?),

            _ => Err(Error::MethodNotFound(method.to_string())).map_err(Into::into),
        }
    }

    fn native_notification(&mut self) -> &mut gdk_common::notification::NativeNotif {
        &mut self.notify
    }

    fn network_parameters(&self) -> &NetworkParameters {
        &self.network_parameters
    }
}

impl GreenlightSession {
    pub fn login(&mut self, login_info: LoginInfo, mnemonic: Mnemonic) -> Result<(), Error> {
        debug!("Starting login node_id:{:?}", login_info.node_id);

        let tls = self.tls_config()?;
        let tls_identity = tls
            .clone()
            .identity(login_info.cert.clone().into_bytes(), login_info.key.clone().into_bytes());

        let node_id = Vec::<u8>::from_hex(&login_info.node_id)?;
        let network_parameters = self.network_parameters();
        let network = network_parameters.network();
        debug!("network is {}", network.to_string());

        let signer = Signer::new(mnemonic.to_seed("").to_vec(), network, tls_identity.clone())?;
        debug!("signer created");

        let client = {
            self.runtime.block_on(async {
                debug!("creating scheduler");
                let scheduler = Scheduler::with(
                    node_id,
                    network,
                    network_parameters.greenlight_url.clone(),
                    &tls,
                )
                .await?;
                scheduler.schedule(tls_identity).await
            })?
        };
        self.client = Some(client);
        debug!("client created");

        let (shutdown_signal, recv) = tokio::sync::mpsc::channel(1);

        let handle: JoinHandle<Result<(), Error>> = std::thread::spawn(move || {
            debug!("Starting signer thread");
            let rt = Builder::new_current_thread().enable_io().enable_time().build()?;
            rt.block_on(async move { signer.run_forever(recv).await })?;
            Ok(())
        });
        debug!("thread spawned");

        self.signer_handle = Some(SignerHandle {
            _handle: handle,
            shutdown_signal,
        });

        self.node_id_hex = Some(login_info.node_id.clone());

        Ok(())
    }
    fn client(&self) -> Result<Client, Error> {
        self.client.as_ref().map(|c| c.clone()).ok_or_else(|| Error::ClientNotInitialized)
    }

    fn cipher(&self) -> Result<&Aes256GcmSiv, Error> {
        self.cipher.as_ref().ok_or_else(|| Error::EncryptionKeysRequired)
    }
    fn tls_config(&self) -> Result<TlsConfig, Error> {
        tls_config(self.network_parameters())
    }

    /// register a greenlight node using the given `secret`
    pub fn register(
        network_parameters: &NetworkParameters,
        mnemonic: &Mnemonic,
    ) -> Result<LoginInfo, Error> {
        let rt = Builder::new_current_thread().enable_io().enable_time().build().unwrap();

        let network = network_parameters.network();

        let signer =
            Signer::new(mnemonic.to_seed("").to_vec(), network, tls_config(&network_parameters)?)?;
        let node_id = signer.node_id();
        let node_id_hex = node_id.to_hex();
        let resp = rt.block_on(async {
            let scheduler = Scheduler::with(
                node_id,
                network,
                network_parameters.greenlight_url.clone(),
                &tls_config(&network_parameters)?,
            )
            .await?;
            scheduler.register(&signer).await
        })?;

        let login_info = LoginInfo {
            cert: resp.device_cert,
            key: resp.device_key,
            node_id: node_id_hex,
        };
        Ok(login_info)
    }

    /// recover an already register greenlight node using the given `secret`, this operations creates
    /// new certificates and must be avoided reusing previous certificate if possible
    pub fn recover(
        network_parameters: &NetworkParameters,
        mnemonic: &Mnemonic,
    ) -> Result<LoginInfo, Error> {
        let rt = Builder::new_current_thread().enable_io().enable_time().build()?;

        let network = network_parameters.network();

        let signer =
            Signer::new(mnemonic.to_seed("").to_vec(), network, tls_config(&network_parameters)?)?;
        let node_id = signer.node_id();
        let node_id_hex = node_id.to_hex();
        let resp = rt.block_on(async {
            let scheduler = Scheduler::with(
                node_id,
                network,
                network_parameters.greenlight_url.clone(),
                &tls_config(&network_parameters)?,
            )
            .await?;
            scheduler.recover(&signer).await
        })?;

        let login_info = LoginInfo {
            cert: resp.device_cert,
            key: resp.device_key,
            node_id: node_id_hex,
        };
        Ok(login_info)
    }

    pub(crate) fn handle_internal_call(
        &mut self,
        method: &str,
        input: Value,
    ) -> Result<Value, Error> {
        match method {
            "authenticate" => {
                let input: AuthenticateInput = serde_json::from_value(input)?;
                Ok(serde_json::to_value(self.authenticate(input)?)?)
            }

            "set_local_encryption_keys" => {
                let input: SetLocalEncryptionKeysInput = serde_json::from_value(input)?;
                self.set_local_encryption_keys(input);
                Ok(Value::Null)
            }
            _ => Err(Error::InternalMethodNotFound(method.to_string())),
        }
    }

    pub fn connect_peer(&mut self, req: ConnectRequest) -> Result<ConnectResponse, Error> {
        Ok(self.runtime.block_on(self.client()?.connect_peer(req))?.into_inner())
    }
    pub fn disconnect(&mut self, req: DisconnectRequest) -> Result<DisconnectResponse, Error> {
        Ok(self.runtime.block_on(self.client()?.disconnect(req))?.into_inner())
    }
    pub fn fund_channel(&mut self, req: FundChannelRequest) -> Result<FundChannelResponse, Error> {
        Ok(self.runtime.block_on(self.client()?.fund_channel(req))?.into_inner())
    }
    pub fn get_info(&mut self) -> Result<GetInfoResponse, Error> {
        let req = GetInfoRequest::default();
        Ok(self.runtime.block_on(self.client()?.get_info(req))?.into_inner())
    }
    pub fn create_invoice(&mut self, mut req: InvoiceRequest) -> Result<Invoice, Error> {
        req.description = req.description.to_lowercase();
        Ok(self.runtime.block_on(self.client()?.create_invoice(req))?.into_inner())
    }
    pub fn keysend(&mut self, req: KeysendRequest) -> Result<Payment, Error> {
        Ok(self.runtime.block_on(self.client()?.keysend(req))?.into_inner())
    }
    pub fn list_funds(&mut self) -> Result<ListFundsResponse, Error> {
        let req = ListFundsRequest::default();
        Ok(self.runtime.block_on(self.client()?.list_funds(req))?.into_inner())
    }
    pub fn list_invoice(&mut self) -> Result<ListInvoicesResponse, Error> {
        let req = ListInvoicesRequest::default();
        Ok(self.runtime.block_on(self.client()?.list_invoices(req))?.into_inner())
    }
    pub fn list_payments(&mut self) -> Result<ListPaymentsResponse, Error> {
        let req = ListPaymentsRequest::default();
        Ok(self.runtime.block_on(self.client()?.list_payments(req))?.into_inner())
    }
    pub fn list_peers(&mut self) -> Result<ListPeersResponse, Error> {
        let req = ListPeersRequest::default();
        Ok(self.runtime.block_on(self.client()?.list_peers(req))?.into_inner())
    }
    pub fn new_addr(&mut self) -> Result<NewAddrResponse, Error> {
        let req = NewAddrRequest::default();
        Ok(self.runtime.block_on(self.client()?.new_addr(req))?.into_inner())
    }
    pub fn pay(&mut self, req: PayRequest) -> Result<Payment, Error> {
        Ok(self.runtime.block_on(self.client()?.pay(req))?.into_inner())
    }
    pub fn withdraw(&mut self, req: WithdrawRequest) -> Result<WithdrawResponse, Error> {
        Ok(self.runtime.block_on(self.client()?.withdraw(req))?.into_inner())
    }

    pub fn set_local_encryption_keys(&mut self, input: SetLocalEncryptionKeysInput) {
        self.cipher = Some(input.xpub.to_cipher().expect("infallible at the moment"));
    }

    /// Retrieve login information (certificate and keys) from the given mnemonic in one of the
    /// following way:
    /// 1) From persisted storage (encrypted)
    /// 2) Using the provided recover server endpoint (save result locally)
    /// 3) Registering the mnemonic with the server endpoint (save result locally)
    /// After recovering certificate and keys it logins with the server endpoint
    pub fn authenticate(&mut self, input: AuthenticateInput) -> Result<AuthenticateOutput, Error> {
        let mut path: PathBuf = self.network_parameters.state_dir.as_str().into();
        if path.exists() && !path.is_dir() {
            return Err(Error::StateDirNotDirectory(path));
        }
        path.push("greenlight_state");
        let _res = fs::create_dir_all(&path);
        path.push("login_info");

        let mnemonic = input.validate()?;

        let exist = path.exists();
        let how;

        let login_info = if exist {
            // read from disk
            how = How::Disk;
            let mut file = File::open(&path)?;

            let plain_text = file.decrypt(self.cipher()?)?;
            let login_info = serde_json::from_slice(&plain_text)?;
            debug!("login_info read from disk {:?} ", &path);
            login_info
        } else {
            // try to recover
            let mut file = File::create(&path)?;
            let login_info = match GreenlightSession::recover(&self.network_parameters, &mnemonic) {
                Ok(login_info) => {
                    debug!("login_info recovered");
                    how = How::Recover;
                    login_info
                }
                Err(_) => {
                    // register
                    debug!("recovered failed, registering");
                    how = How::Register;
                    GreenlightSession::register(&self.network_parameters, &mnemonic)?
                }
            };
            let data = serde_json::to_vec(&login_info)?;
            let (nonce, cipher_text) = data.encrypt(self.cipher()?)?;
            file.write_all(&nonce[..])?;
            file.write_all(&cipher_text[..])?;

            debug!("login_info written to disk {:?}", path);

            login_info
        };

        debug!("authenticate, credentials: {:?}", input);

        self.login(login_info, input.validate()?)?;
        Ok(AuthenticateOutput {
            how,
        })
    }
}

fn tls_config(n: &NetworkParameters) -> Result<TlsConfig, Error> {
    Ok(TlsConfig::with(&n.nobody_crt, &n.nobody_key, &n.ca_crt)?)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetLocalEncryptionKeysInput {
    pub xpub: ExtendedPubKey,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticateInput {
    pub mnemonic: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticateOutput {
    pub how: How,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum How {
    Disk,
    Recover,
    Register,
}

impl AuthenticateInput {
    fn validate(&self) -> Result<Mnemonic, Error> {
        Ok(Mnemonic::from_str(&self.mnemonic)?)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use bip39::Mnemonic;
    use bitcoin::Network;
    use gdk_test::greenlight::network_parameters;
    use gl_client::pb::{amount, Amount};
    use gl_client::signer::Signer;

    #[test]
    fn test_amount() {
        let amount = Amount {
            unit: Some(amount::Unit::Satoshi(1000)),
        };
        assert_eq!("{\"unit\":{\"Satoshi\":1000}}", serde_json::to_string(&amount).unwrap());
        //let amount: Amount = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn test_derivation() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let mnemonic = Mnemonic::parse(mnemonic).unwrap();
        let (network_parameters, _dir) = network_parameters(false);
        assert_eq!(network_parameters.network(), Network::Regtest);
        let seed = mnemonic.to_seed("");
        assert_eq!(seed.to_hex(), "5eb00bbddcf069084889a8ab9155568165f5c453ccb85e70811aaed6f6da5fc19a5ac40b389cd370d086206dec8aa6c43daea6690f20ad3d8d48b2d2ce9e38e4");

        let signer =
            Signer::new(seed.to_vec(), Network::Regtest, tls_config(&network_parameters).unwrap())
                .unwrap();
        let _node_id = signer.node_id();

        // The following doesn't work:  // node_id.to_hex() is "03653e90c1ce4660fd8505dd6d643356e93cfe202af109d382787639dd5890e87d"
        // greenlight client use libhsmd to calculate via c_init(), maybe wait for VLS integration

        // assert_eq!(
        //     "025822aa0d1afff06d21ca9dbc76d1e6ee3e3fc750971ae47f8b33b98c730b7067",
        //     node_id.to_hex()
        // );

        // why? Got from the test vector from VLS test

        // let node_config = NodeConfig {
        //     network: bitcoin::Network::Regtest,
        //     key_derivation_style: KeyDerivationStyle::Native,
        // };

        // let persister: Arc<dyn Persist> = Arc::new(DummyPersister);
        // let validator_factory = Arc::new(SimpleValidatorFactory::new());

        // let node = Node::new(node_config, &seed, &persister, vec![], validator_factory);

        // let node_id = node.get_id();

        // or

        // echo "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" | vls-cli node new -m --network regtest

        // let node_private_bytes = hkdf_sha256(seed, "nodeid".as_bytes(), &[]);
        // let node_secret_key = SecretKey::from_slice(&node_private_bytes).unwrap();
        // let node_id = PublicKey::from_secret_key(&secp_ctx, &node_secret_key);
        // (node_id, node_secret_key)
    }
}
