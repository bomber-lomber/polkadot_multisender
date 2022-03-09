mod cli;
mod transfer;

use clap::Parser;
use futures::StreamExt;
use sp_keyring::AccountKeyring;
use subxt::{
    sp_runtime::MultiAddress, ClientBuilder, DefaultConfig, DefaultExtra, PairSigner,
    TransactionStatus,
};

use crate::transfer::TransferInfo;

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod polkadot {}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = cli::Args::parse();
    let api = ClientBuilder::new()
        .build()
        .await?
        .to_runtime_api::<polkadot::RuntimeApi<DefaultConfig, DefaultExtra<DefaultConfig>>>();
    let signer = PairSigner::new(
        AccountKeyring::from_account_id(&args.sender)
            .expect("sender account doesn't exists")
            .pair(),
    );
    let mut reader = csv::Reader::from_path(&args.csv_file)?;
    let mut line_counter = 1;
    for item in reader.deserialize() {
        let info: TransferInfo = item?;
        let dest = MultiAddress::Id(info.destination_account_id);
        println!("Transfer #{line_counter} start... ");
        let balance_transfer_progress = api
            .tx()
            .balances()
            .transfer(dest, info.amount)
            .sign_and_submit_then_watch(&signer)
            .await?;
        handle_transfer_events_loop(balance_transfer_progress).await?;
        line_counter += 1;
        println!("... finished.");
        println!();
    }
    Ok(())
}

async fn handle_transfer_events_loop(
    mut balance_transfer_progress: subxt::TransactionProgress<
        '_,
        subxt::DefaultConfig,
        polkadot::runtime_types::sp_runtime::DispatchError,
        polkadot::Event,
    >,
) -> Result<(), Box<dyn std::error::Error>> {
    while let Some(ev) = balance_transfer_progress.next().await {
        let ev = ev?;

        if let TransactionStatus::InBlock(details) = ev {
            println!(
                "Transaction {:?} made it into block {:?} ...",
                details.extrinsic_hash(),
                details.block_hash()
            );

            let events = details.wait_for_success().await?;
            let transfer_event = events.find_first::<polkadot::balances::events::Transfer>()?;

            if let Some(event) = transfer_event {
                println!(
                    "Balance transfer is now in block (but not finalized): {:?}",
                    event
                );
            } else {
                println!("Failed to find Balances::Transfer Event");
            }
        } else if let TransactionStatus::Finalized(details) = ev {
            println!(
                "Transaction {:?} is finalized in block {:?}",
                details.extrinsic_hash(),
                details.block_hash()
            );

            let events = details.wait_for_success().await?;
            let transfer_event = events.find_first::<polkadot::balances::events::Transfer>()?;

            if let Some(event) = transfer_event {
                println!("Balance transfer success: {:?}", event);
            } else {
                println!("Failed to find Balances::Transfer Event");
            }
        } else {
            println!("Current transaction status: {:?}", ev);
        }
    }
    Ok(())
}
