use crate::node::Node;
use anyhow::anyhow;
use anyhow::Result;
use dlc_manager::Oracle;
use dlc_manager::Wallet;
use serde::Serialize;
use std::str::FromStr;
use std::time::Duration;

mod add_dlc;
mod channel_less_payment;
mod dlc_collaborative_settlement;
mod dlc_non_collaborative_settlement;
mod multi_hop_payment;
mod single_hop_payment;

const CHOPSTICKS_FAUCET_ORIGIN: &str = "http://localhost:3000";

const ELECTRS_ORIGIN: &str = "tcp://localhost:50000";

struct MockOracle;

impl Oracle for MockOracle {
    fn get_public_key(&self) -> bitcoin::XOnlyPublicKey {
        bitcoin::XOnlyPublicKey::from_str(
            "18845781f631c48f1c9709e23092067d06837f30aa0cd0544ac887fe91ddd166",
        )
        .unwrap()
    }

    fn get_announcement(
        &self,
        _event_id: &str,
    ) -> Result<dlc_messages::oracle_msgs::OracleAnnouncement, dlc_manager::error::Error> {
        todo!()
    }

    fn get_attestation(
        &self,
        _event_id: &str,
    ) -> Result<dlc_messages::oracle_msgs::OracleAttestation, dlc_manager::error::Error> {
        todo!()
    }
}

impl Node {
    async fn fund(&self, amount: bitcoin::Amount) -> Result<()> {
        let starting_balance = self.get_confirmed_balance()?;
        let expected_balance = starting_balance + amount.to_sat();

        let address = self
            .wallet
            .get_new_address()
            .map_err(|_| anyhow!("Failed to get new address"))?;

        fund_and_mine(address, amount).await;

        while self.get_confirmed_balance()? < expected_balance {
            let interval = Duration::from_millis(200);

            self.sync();

            tokio::time::sleep(interval).await;
            tracing::debug!(
                ?interval,
                "Checking if wallet has been funded after interval"
            )
        }

        Ok(())
    }

    fn get_confirmed_balance(&self) -> Result<u64> {
        let balance = self.wallet.inner().get_balance()?;

        Ok(balance.confirmed)
    }
}

/// Instructs `nigiri-chopsticks` to mine a block and spend the given `amount` from the coinbase
/// transaction to the given `address`.
async fn fund_and_mine(address: bitcoin::Address, amount: bitcoin::Amount) {
    #[derive(Serialize)]
    struct Payload {
        address: bitcoin::Address,
        #[serde(with = "bdk::bitcoin::util::amount::serde::as_btc")]
        amount: bitcoin::Amount,
    }

    let client = reqwest::Client::new();

    let result = client
        .post(format!("{CHOPSTICKS_FAUCET_ORIGIN}/faucet"))
        .json(&Payload { address, amount })
        .send()
        .await
        .unwrap();

    assert!(result.status().is_success());
}
