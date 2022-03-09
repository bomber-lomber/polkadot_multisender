use clap::Parser;
use subxt::sp_runtime::AccountId32;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long)]
    pub sender: AccountId32,

    #[clap(short, long)]
    pub csv_file: String,
}
