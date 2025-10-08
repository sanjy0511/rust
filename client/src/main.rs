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
    io::{self, Write},
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
        println!("\n1. Signup\n2. Signin\n3. List Users\n4. Exit");
        print!("Enter choice: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => signup(&client, &payer, &program_id),
            "2" => signin(&client, &payer, &program_id),
            "3" => list_users_onchain(&client, &program_id),
            "4" => {
                println!("Exiting...!");
                break;
            }
            _ => println!("Invalid choice"),
        }
    }
}

// ---------------- Signup ----------------
fn signup(client: &RpcClient, payer: &solana_sdk::signer::keypair::Keypair, program_id: &Pubkey) {
    let (username, email, password) = get_signup_details();

    let (user_pda, _bump) = Pubkey::find_program_address(&[username.as_bytes()], program_id);
    let (registry_pda, _bump_registry) = Pubkey::find_program_address(&[b"registry"], program_id);

    let ix = Instruction::new_with_bytes(
        *program_id,
        &UserInstruction::Signup {
            username: username.clone(),
            email: email.clone(),
            password: password.clone(),
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

    let recent_blockhash = client.get_latest_blockhash().unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );

    match client.send_and_confirm_transaction(&tx) {
        Ok(sig) => println!("Signup success! Tx: {}", sig),
        Err(err) => println!("Signup failed: {:?}", err),
    }
}

// ---------------- Signin ----------------
fn signin(client: &RpcClient, payer: &solana_sdk::signer::keypair::Keypair, program_id: &Pubkey) {
    let (username, password) = get_signin_details();
    let (user_pda, _bump) = Pubkey::find_program_address(&[username.as_bytes()], program_id);

    let ix = Instruction::new_with_bytes(
        *program_id,
        &UserInstruction::Signin {
            username: username.clone(),
            password: password.clone(),
        }
        .try_to_vec()
        .unwrap(),
        vec![AccountMeta::new(user_pda, false)],
    );

    let recent_blockhash = client.get_latest_blockhash().unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );

    match client.send_and_confirm_transaction(&tx) {
        Ok(sig) => println!(" Signin Tx sent: {}", sig),
        Err(err) => println!(" Signin failed: {:?}", err),
    }
}

// ---------------- List Users ----------------
fn list_users_onchain(client: &RpcClient, program_id: &Pubkey) {
    let (registry_pda, _bump_registry) = Pubkey::find_program_address(&[b"registry"], program_id);

    println!("Fetching users from registry PDA: {}", registry_pda);

    let account_data = match client.get_account_data(&registry_pda) {
        Ok(data) => data,
        Err(_) => {
            println!(" No registry found on-chain. No users yet.");
            return;
        }
    };

    // Safely attempt to deserialize registry
    let registry: UserRegistry = match UserRegistry::try_from_slice(&account_data) {
        Ok(reg) => reg,
        Err(err) => {
            println!(" Failed to deserialize registry account: {:?}", err);
            return;
        }
    };

    if registry.users.is_empty() {
        println!("No users signed up yet.");
        return;
    }

    println!("--- Registered Users On-Chain ---");
    for (index, user_pda) in registry.users.iter().enumerate() {
        let user_data = match client.get_account_data(user_pda) {
            Ok(data) => data,
            Err(_) => {
                println!(" Could not fetch user account {}", user_pda);
                continue;
            }
        };

        match UserAccount::try_from_slice(&user_data) {
            Ok(user) => println!(
                "{}. Username: {}, Email: {}",
                index + 1,
                user.username,
                user.email
            ),
            Err(_) => println!(" Failed to deserialize user account {}", user_pda),
        }
    }
}

// ---------------- Helper Functions ----------------
fn get_signup_details() -> (String, String, String) {
    let mut username = String::new();
    let mut email = String::new();
    let mut password = String::new();

    print!("Enter username: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut username).unwrap();
    print!("Enter email: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut email).unwrap();
    print!("Enter password: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut password).unwrap();

    (
        username.trim().to_string(),
        email.trim().to_string(),
        password.trim().to_string(),
    )
}

fn get_signin_details() -> (String, String) {
    let mut username = String::new();
    let mut password = String::new();

    print!("Enter username: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut username).unwrap();
    print!("Enter password: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut password).unwrap();

    (username.trim().to_string(), password.trim().to_string())
}
