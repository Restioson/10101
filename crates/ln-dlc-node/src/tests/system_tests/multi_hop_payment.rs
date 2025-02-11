use crate::tests::system_tests::create_tmp_dir;
use crate::tests::system_tests::fund_and_mine;
use crate::tests::system_tests::has_channel;
use crate::tests::system_tests::init_tracing;
use crate::tests::system_tests::log_channel_id;
use crate::tests::system_tests::setup_ln_node;
use dlc_manager::Wallet;
use std::time::Duration;

#[tokio::test]
async fn multi_hop_payment() {
    init_tracing();

    let test_dir = create_tmp_dir("multi_hop_test");

    // 1. Set up three LN-DLC nodes.
    let alice = setup_ln_node(&test_dir, "alice", false).await;
    let bob = setup_ln_node(&test_dir, "bob", true).await;
    let claire = setup_ln_node(&test_dir, "claire", false).await;

    tracing::info!("Alice: {}", alice.info);
    tracing::info!("Bob: {}", bob.info);
    tracing::info!("Claire: {}", claire.info);

    let _alice_bg = alice.start().await.unwrap();
    let _bob_bg = bob.start().await.unwrap();
    let _claire_bg = claire.start().await.unwrap();

    // 2. Connect the two nodes.

    // TODO: Remove sleep by allowing the first connection attempt to be retried
    tokio::time::sleep(Duration::from_secs(2)).await;
    alice.keep_connected(bob.info).await.unwrap();
    claire.keep_connected(bob.info).await.unwrap();
    alice.keep_connected(claire.info).await.unwrap();

    // 3. Fund the Bitcoin wallets of the nodes who will open a channel.
    {
        alice
            .fund(bitcoin::Amount::from_sat(1_000_000))
            .await
            .unwrap();
        bob.fund(bitcoin::Amount::from_sat(1_000_000))
            .await
            .unwrap();

        // we need to wait here for the wallet to sync properly
        tokio::time::sleep(Duration::from_secs(5)).await;

        alice.sync();
        let balance = alice.wallet.inner().get_balance().unwrap();
        tracing::info!(%balance, "Alice's wallet balance after calling the faucet");

        bob.sync();
        let balance = bob.wallet.inner().get_balance().unwrap();
        tracing::info!(%balance, "Bob's wallet balance after calling the faucet");

        claire.sync();
        let balance = claire.wallet.inner().get_balance().unwrap();
        tracing::info!(%balance, "Claire's wallet balance after calling the faucet");
    }

    tracing::info!("Opening channel");

    // 4. Create channel between alice and bob.
    alice.open_channel(bob.info, 30000, 0).unwrap();
    // 4. Create channel between bob and claire.
    bob.open_channel(claire.info, 30000, 0).unwrap();

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Add 1 confirmation required for the channel to get usable.
    let address = alice.wallet.get_new_address().unwrap();
    fund_and_mine(address, bitcoin::Amount::from_sat(1000)).await;

    // Add 5 confirmations for the channel to get announced.
    for _ in 1..6 {
        let address = alice.wallet.get_new_address().unwrap();
        fund_and_mine(address, bitcoin::Amount::from_sat(1000)).await;
    }

    tokio::time::sleep(Duration::from_secs(2)).await;

    // TODO: it would be nicer if we could hook that assertion to the corresponding event received
    // through the event handler.
    loop {
        alice.sync();
        bob.sync();
        claire.sync();

        tracing::debug!("Checking if channel is open yet");

        if has_channel(&alice, &bob) && has_channel(&bob, &claire) {
            break;
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    tracing::info!("Channel open");

    log_channel_id(&alice, 0, "alice-bob");
    log_channel_id(&bob, 0, "bob-alice");
    log_channel_id(&bob, 1, "bob-claire");
    log_channel_id(&claire, 0, "claire-bob");

    // 5. Generate an invoice from the payer to the payee.
    let invoice_amount = 500;
    let invoice = claire.create_invoice(invoice_amount).unwrap();

    alice.sync();
    bob.sync();
    claire.sync();

    tracing::info!(?invoice);

    tokio::time::sleep(Duration::from_secs(5)).await;

    // 6. Pay the invoice.
    alice.send_payment(&invoice).unwrap();

    tokio::time::sleep(Duration::from_secs(5)).await;

    alice.sync();
    let balance = alice.get_ldk_balance().unwrap();
    tracing::info!(?balance, "Alice's wallet balance");

    bob.sync();
    let balance = bob.get_ldk_balance().unwrap();
    tracing::info!(?balance, "Bob's wallet balance");

    claire.sync();
    let balance = claire.get_ldk_balance().unwrap();
    tracing::info!(?balance, "Claire's wallet balance");

    assert_eq!(balance.available, invoice_amount)
}
