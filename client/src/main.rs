use borsh::{BorshDeserialize, BorshSerialize};
use dotenv::dotenv;
use my_counter::{UserAccount, UserInstruction, UserRegistry};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Signer},
    system_program,
    transaction::Transaction,
};
use std::{
    env,
    io::{self, Cursor, Write},
};

fn main() {
    dotenv().ok();
    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");
    let program_id: Pubkey = env::var("PROGRAM_ID")
        .expect("PROGRAM_ID must be set")
        .parse()
        .unwrap();
    let payer_path = env::var("PAYER_PATH").expect("PAYER_PATH must be set");

    let client = RpcClient::new(rpc_url);
    let payer = read_keypair_file(payer_path).unwrap();

    loop {
        println!("\n=== MENU ===");
        println!("1. Signup");
        println!("2. Signin");
        println!("3. List Users");
        println!("4. Create Currency");
        println!("5. Create User Token Account");
        println!("6. Mint To User");
        println!("7. Transfer To User");
        println!("8. Exit");
        print!("Enter choice: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => signup(&client, &payer, &program_id),
            "2" => signin(&client, &payer, &program_id),
            "3" => list_users_onchain(&client, &program_id),
            "4" => create_currency(&client, &payer, &program_id),
            "5" => create_user_token_account(&client, &payer, &program_id),
            "6" => mint_to_user(&client, &payer, &program_id),
            "7" => transfer_to_user(&client, &payer, &program_id),
            "8" => {
                println!("Exiting...");
                break;
            }
            _ => println!("Invalid choice"),
        }
    }
}

// ---------------- SIGNUP ----------------
fn signup(client: &RpcClient, payer: &solana_sdk::signer::keypair::Keypair, program_id: &Pubkey) {
    let (username, email, password) = get_signup_details();
    let (user_pda, _) = Pubkey::find_program_address(&[username.as_bytes()], program_id);
    let (registry_pda, _) = Pubkey::find_program_address(&[b"registry"], program_id);

    let ix = Instruction::new_with_bytes(
        *program_id,
        &UserInstruction::Signup {
            username,
            email,
            password,
        }
        .try_to_vec()
        .unwrap(),
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(user_pda, false),
            AccountMeta::new(registry_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    send_tx(client, payer, &[ix]);
}

// ---------------- SIGNIN ----------------
fn signin(client: &RpcClient, _payer: &solana_sdk::signer::keypair::Keypair, program_id: &Pubkey) {
    let (username, password) = get_signin_details();
    let (user_pda, _) = Pubkey::find_program_address(&[username.as_bytes()], program_id);

    let ix = Instruction::new_with_bytes(
        *program_id,
        &UserInstruction::Signin { username, password }
            .try_to_vec()
            .unwrap(),
        vec![AccountMeta::new(user_pda, false)],
    );

    send_tx(client, _payer, &[ix]);
}

// ---------------- LIST USERS ----------------
fn list_users_onchain(client: &RpcClient, program_id: &Pubkey) {
    let (registry_pda, _) = Pubkey::find_program_address(&[b"registry"], program_id);
    println!("Fetching users from registry PDA: {}", registry_pda);

    let account_data = match client.get_account_data(&registry_pda) {
        Ok(data) => data,
        Err(_) => {
            println!("No registry found on-chain. No users yet.");
            return;
        }
    };

    let mut cursor = Cursor::new(&account_data);

    let registry: UserRegistry = match UserRegistry::deserialize_reader(&mut cursor) {
        Ok(r) => r,
        Err(err) => {
            println!("Failed to deserialize registry account: {:?}", err);
            return;
        }
    };

    if registry.users.is_empty() {
        println!("No users registered yet.");
        return;
    }

    println!("--- Registered Users ---");
    for (index, user_pda) in registry.users.iter().enumerate() {
        match client.get_account_data(user_pda) {
            Ok(user_data) => match UserAccount::try_from_slice(&user_data) {
                Ok(user) => println!(
                    "{}. Username: {}, Email: {}",
                    index + 1,
                    user.username,
                    user.email
                ),
                Err(_) => println!("  Failed to deserialize user account {}", user_pda),
            },
            Err(_) => println!("  Could not fetch user account {}", user_pda),
        }
    }
}

// ---------------- TOKEN LOGIC ----------------
fn create_currency(
    client: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    program_id: &Pubkey,
) {
    print!("Enter total supply (e.g., 100000): ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let total_supply: u64 = input.trim().parse().unwrap_or(100000);

    let mint = solana_sdk::signature::Keypair::new();
    println!("Mint Pubkey: {}", mint.pubkey());

    let token_program_pubkey = Pubkey::new_from_array(spl_token::id().to_bytes());
    let rent_sysvar_pubkey = solana_sdk::sysvar::rent::id();

    let ix = Instruction::new_with_bytes(
        *program_id,
        &UserInstruction::CreateCurrency { total_supply }
            .try_to_vec()
            .unwrap(),
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(mint.pubkey(), true),
            AccountMeta::new_readonly(token_program_pubkey, false),
            AccountMeta::new_readonly(rent_sysvar_pubkey, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    send_tx_multi_signer(client, &[payer, &mint], &[ix]);
}

// ---------------- CREATE USER TOKEN ACCOUNT ----------------
fn create_user_token_account(
    client: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    program_id: &Pubkey,
) {
    let username = get_input("Enter username: ");
    let (user_pda, _bump) = Pubkey::find_program_address(&[username.as_bytes()], program_id);
    let (user_token_pda, _token_bump) =
        Pubkey::find_program_address(&[username.as_bytes(), b"token"], program_id);
    let mint_pubkey: Pubkey = get_input("Enter Mint Pubkey: ").parse().unwrap();

    let ix = Instruction::new_with_bytes(
        *program_id,
        &UserInstruction::CreateUserTokenAccount {
            username: username.clone(),
        }
        .try_to_vec()
        .unwrap(),
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(user_token_pda, false),
            AccountMeta::new(mint_pubkey, false),
            AccountMeta::new_readonly(Pubkey::new_from_array(spl_token::id().to_bytes()), false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(
                Pubkey::new_from_array(solana_sdk::sysvar::rent::id().to_bytes()),
                false,
            ),
            AccountMeta::new(user_pda, false), // user PDA
        ],
    );

    println!("Sending transaction...");
    send_tx(client, payer, &[ix]);
}

// ---------------- MINT ----------------
fn mint_to_user(
    client: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    program_id: &Pubkey,
) {
    let username = get_input("Enter username: ");
    let mint_pubkey: Pubkey = get_input("Enter Mint Pubkey: ").parse().unwrap();
    let user_token_account: Pubkey = get_input("Enter User Token Account Pubkey: ")
        .parse()
        .unwrap();
    let amount: u64 = get_input("Enter amount to mint: ").parse().unwrap();

    let token_program_pubkey = Pubkey::new_from_array(spl_token::id().to_bytes());

    let ix = Instruction::new_with_bytes(
        *program_id,
        &UserInstruction::MintToUser { username, amount }
            .try_to_vec()
            .unwrap(),
        vec![
            AccountMeta::new(mint_pubkey, false),
            AccountMeta::new(user_token_account, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(token_program_pubkey, false),
        ],
    );

    send_tx(client, payer, &[ix]);
}

// ---------------- TRANSFER ----------------
fn transfer_to_user(
    client: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    program_id: &Pubkey,
) {
    let from_token_account: Pubkey = get_input("Enter From User Token Account Pubkey: ")
        .parse()
        .unwrap();
    let to_token_account: Pubkey = get_input("Enter To User Token Account Pubkey: ")
        .parse()
        .unwrap();
    let amount: u64 = get_input("Enter amount to transfer: ").parse().unwrap();

    let token_program_pubkey = Pubkey::new_from_array(spl_token::id().to_bytes());

    let ix = Instruction::new_with_bytes(
        *program_id,
        &UserInstruction::TransferToUser {
            from_username: "".to_string(),
            to_username: "".to_string(),
            amount,
        }
        .try_to_vec()
        .unwrap(),
        vec![
            AccountMeta::new(from_token_account, false),
            AccountMeta::new(to_token_account, false),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new_readonly(token_program_pubkey, false),
        ],
    );

    send_tx(client, payer, &[ix]);
}

// ---------------- COMMON TX HELPERS ----------------
fn send_tx(client: &RpcClient, payer: &solana_sdk::signer::keypair::Keypair, ix: &[Instruction]) {
    let bh = client.get_latest_blockhash().unwrap();
    let tx = Transaction::new_signed_with_payer(ix, Some(&payer.pubkey()), &[payer], bh);
    match client.send_and_confirm_transaction(&tx) {
        Ok(sig) => println!("Transaction sent successfully: {}", sig),
        Err(e) => println!("Error sending transaction: {:?}", e),
    }
}

fn send_tx_multi_signer(
    client: &RpcClient,
    signers: &[&solana_sdk::signer::keypair::Keypair],
    ix: &[Instruction],
) {
    let bh = client.get_latest_blockhash().unwrap();
    let tx = Transaction::new_signed_with_payer(ix, Some(&signers[0].pubkey()), signers, bh);
    match client.send_and_confirm_transaction(&tx) {
        Ok(sig) => println!("Transaction sent successfully: {}", sig),
        Err(e) => println!("Error sending transaction: {:?}", e),
    }
}

// ---------------- INPUT HELPERS ----------------
fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn get_signup_details() -> (String, String, String) {
    let username = get_input("Username: ");
    let email = get_input("Email: ");
    let password = get_input("Password: ");
    (username, email, password)
}

fn get_signin_details() -> (String, String) {
    let username = get_input("Username: ");
    let password = get_input("Password: ");
    (username, password)
}
