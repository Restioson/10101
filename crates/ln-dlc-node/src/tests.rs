use dlc_manager::Oracle;
use serde::Serialize;
use std::str::FromStr;

mod add_dlc;
mod channel_less_payment;
mod dlc_collaborative_settlement;
mod dlc_non_collaborative_settlement;
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

async fn fund_and_mine(address: bitcoin::Address, amount: bitcoin::Amount) {
    #[derive(Serialize)]
    struct Payload {
        address: bitcoin::Address,
        #[serde(with = "bdk::bitcoin::util::amount::serde::as_btc")]
        amount: bitcoin::Amount,
    }

    let client = reqwest::Client::new();
    // mines a block and spends the given amount from the coinbase transaction to the given address
    let result = client
        .post(format!("{CHOPSTICKS_FAUCET_ORIGIN}/faucet"))
        .json(&Payload { address, amount })
        .send()
        .await
        .unwrap();

    assert!(result.status().is_success());
}
