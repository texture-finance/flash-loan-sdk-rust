use std::path::PathBuf;

use derive_more::FromStr;
use num_traits::pow;
use solana_client::client_error::ClientError;
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::{read_keypair_file, Signature};
use solana_sdk::signer::Signer;
use solana_sdk::signers::Signers;
use solana_sdk::transaction::Transaction;
use structopt::StructOpt;

use flash_loan_sdk::instruction::{flash_borrow, flash_repay};
use flash_loan_sdk::{available_liquidity, flash_loan_fee, get_reserve, FLASH_LOAN_ID};

fn main() {
    let opt = Opts::from_args();

    println!("=====================Setup=====================");
    println!("Solana cluster       : {}", opt.url);
    println!("Flash loan program id: {}", opt.program_id);
    println!("Flash loan reserve   : {}", opt.reserve);
    println!("===============================================");

    let rpc_client = RpcClient::new_with_commitment(opt.url.clone(), CommitmentConfig::confirmed());

    // From Solana RPC rate limit perspective it is more efficient to load Reserve once from the chain and then
    // use it in subsequent calls.
    let reserve = get_reserve(&opt.reserve, &rpc_client).expect("Getting reserve");

    // All token amounts in this SDK are in lamports (or equivalent fractional token units).
    // To calculate fractional units from human readable amount do this...
    let amount_to_borrow = pow(10, reserve.liquidity.mint_decimals as usize) * 10; // Gives 10 SOL amount expressed as lamports

    let available_liquidity = available_liquidity(&reserve);
    println!("Available liquidity: {} lamports", available_liquidity);

    // We need to return amount_to_borrow + fee at the end of the flash loan transaction. So its worth
    // to make sure that we have enough money to pay fees. Get the fees amount via flash_loan_fee() call.
    let fee = flash_loan_fee(&reserve, amount_to_borrow).expect("Calculating fee");
    println!(
        "Fee to borrow {} lamports will be: {} lamports",
        amount_to_borrow, fee
    );

    // Construct FlashBorrow instruction. Here we specify amount_to_borrow without fees.
    let flash_borrow_ix = flash_borrow(
        opt.program_id,
        amount_to_borrow,
        reserve.liquidity.supply_pubkey,
        opt.wallet,
        opt.reserve,
        reserve.lending_market,
    );

    let authority_kp = read_keypair_file(opt.authority.0).expect("Reading authority key pair file");

    // Construct FlashRepay instruction. Again we specify amount_to_borrow without fees.
    // But when contract will be executing this IX it will transfer amount_to_borrow + fee from user's wallet!
    let flash_repay_ix = flash_repay(
        opt.program_id,
        amount_to_borrow,
        opt.wallet,
        reserve.liquidity.supply_pubkey,
        reserve.config.fee_receiver,
        opt.reserve,
        reserve.lending_market,
        authority_kp.pubkey(),
    );

    // Put FlashBorrow first and FlashRepay thereafter. This is simplified example. In real world
    // applications there will be other instructions in between (e.g. swaps on DEXes).
    // Those instructions will be able to use borrowed tokens in their logic.
    sign_and_send_transaction(
        &authority_kp.pubkey(),
        &[
            flash_borrow_ix,
            /* IXes which use borrowed amount go here*/ flash_repay_ix,
        ],
        &[&authority_kp],
        &rpc_client,
    )
    .expect("Sending TX");

    println!("Successfully flash borrowed!");
}

fn sign_and_send_transaction(
    payer: &Pubkey,
    ixs: impl AsRef<[Instruction]>,
    signers: &impl Signers,
    rpc_client: &RpcClient,
) -> Result<Signature, ClientError> {
    let mut tx = Transaction::new_with_payer(ixs.as_ref(), Some(payer));
    let blockhash = rpc_client.get_latest_blockhash()?;

    tx.sign(signers, blockhash);

    println!("Sending transaction {:?}", tx);

    let signature = rpc_client.send_and_confirm_transaction_with_spinner(&tx)?;

    println!("Signature: {}", signature);
    Ok(signature)
}

#[derive(StructOpt)]
/// This is Flash Loan SDK example. By default it connects to Solana Devnet claster with
/// preconfigured Flash Loan program and Reserves. Namely this program works with wrapped
/// SOL reserve.
#[structopt(rename_all = "kebab-case")]
pub struct Opts {
    /// Flash Loan program id. Defaults to Devnet address.
    #[structopt(long, default_value = FLASH_LOAN_ID)]
    pub program_id: Pubkey,

    /// Keypair to use for signing instructions (e.g. authorise transfers from wallet) and pay fees.
    #[structopt(long, short, default_value)]
    pub authority: KeypairPath,

    /// User's SPL Token wallet of native mint (aka wrapped SOL)
    #[structopt(long, short)]
    pub wallet: Pubkey,

    /// Solana RPC endpoint to work with
    #[structopt(long, short, default_value = "https://api.devnet.solana.com")]
    pub url: String,

    /// Flash Loan Reserve to work with.
    #[structopt(long, short)]
    pub reserve: Pubkey,
}

#[derive(FromStr)]
pub struct KeypairPath(pub PathBuf);

impl Default for KeypairPath {
    fn default() -> Self {
        let mut path = dirs_next::home_dir().expect("home dir");
        path.extend(&[".config", "solana", "id.json"]);
        Self(path)
    }
}

impl ToString for KeypairPath {
    fn to_string(&self) -> String {
        self.0.to_str().expect("non unicode").to_string()
    }
}
