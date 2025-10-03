use borsh::{BorshDeserialize, BorshSerialize};
use dotenv::dotenv;
use my_counter::{UserAccount, UserInstruction};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Signer},
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
        let choice = choice.trim();

        match choice {
            "1" => signup_user(&client, &payer, &program_id),
            "2" => signin_user(&client, &program_id),
            "3" => list_users(&client, &program_id),
            "4" => break,
            _ => println!("Invalid choice!"),
        }
    }
}

// ---------------- Signup ----------------
fn signup_user(
    client: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    program_id: &Pubkey,
) {
    let (username, email, password) = get_signup_details();
    let (pda, _bump) = Pubkey::find_program_address(&[username.as_bytes()], program_id);

    let signup_ix = Instruction::new_with_bytes(
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
            AccountMeta::new(pda, false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
    );

    send_transaction(client, payer, &[signup_ix]);
}

// ---------------- Signin ----------------
fn signin_user(client: &RpcClient, program_id: &Pubkey) {
    let (username, password) = get_signin_details();
    let (pda, _bump) = Pubkey::find_program_address(&[username.as_bytes()], program_id);

    match client.get_account_data(&pda) {
        Ok(data) => {
            if let Ok(user_account) = UserAccount::try_from_slice(&data) {
                if user_account.username == username && user_account.password == password {
                    println!("Signin success! Welcome, {}", username);
                } else {
                    println!("Signin failed: Invalid username or password");
                }
            } else {
                println!("Signin failed: Account data corrupted");
            }
        }
        Err(_) => println!("Signin failed: User not found, please signup first"),
    }
}

// ---------------- List Users ----------------
fn list_users(client: &RpcClient, program_id: &Pubkey) {
    match client.get_program_accounts(program_id) {
        Ok(accounts) => {
            if accounts.is_empty() {
                println!("No users found.");
            } else {
                println!("Registered users:");
                for (pubkey, account) in accounts {
                    if let Ok(user) = UserAccount::try_from_slice(&account.data) {
                        println!(
                            "Username: {}, Email: {}, PDA: {}",
                            user.username, user.email, pubkey
                        );
                    } else {
                        println!("Failed to deserialize account: {}", pubkey);
                    }
                }
            }
        }
        Err(err) => println!("Failed to fetch users: {:?}", err),
    }
}

// ---------------- Helpers ----------------
fn send_transaction(
    client: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    ixs: &[Instruction],
) {
    let mut tx = Transaction::new_with_payer(ixs, Some(&payer.pubkey()));
    tx.sign(&[payer], client.get_latest_blockhash().unwrap());
    match client.send_and_confirm_transaction(&tx) {
        Ok(sig) => println!("Transaction success! Tx: {}", sig),
        Err(e) => println!("Transaction failed: {:?}", e),
    }
}

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
