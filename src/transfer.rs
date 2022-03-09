use serde::Deserialize;
use subxt::sp_runtime::AccountId32;

#[derive(Deserialize, Debug)]
pub struct TransferInfo {
    pub destination_account_id: AccountId32,
    pub amount: u128,
}
