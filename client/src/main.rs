use borsh::BorshSerialize;
use dotenv::dotenv;
use my_counter::UserInstruction;
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

// ---------------- Temporary DB Struct ----------------
#[derive(Debug, Clone)]
struct TempUser {
    username: String,
    email: String,
    password: String,
}

// ---------------- MAIN ----------------
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

    // Local in-memory DB
    let mut temp_db: Vec<TempUser> = Vec::new();

    loop {
        println!("\n1. Signup\n2. Signin\n3. List Users\n4. Exit");
        print!("Enter choice: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        match choice.trim() {
            "1" => signup(&client, &payer, &program_id, &mut temp_db),
            "2" => signin(&client, &payer, &program_id, &temp_db),
            "3" => list_users(&temp_db),
            "4" => break,
            _ => println!("Invalid choice"),
        }
    }
}

// ---------------- Signup ----------------
fn signup(
    client: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    program_id: &Pubkey,
    temp_db: &mut Vec<TempUser>,
) {
    let (username, email, password) = get_signup_details();
    let (pda, _bump) = Pubkey::find_program_address(&[username.as_bytes()], program_id);

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
            AccountMeta::new(pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    tx.sign(&[payer], client.get_latest_blockhash().unwrap());

    match client.send_and_confirm_transaction(&tx) {
        Ok(sig) => {
            println!("Signup success! Tx: {}", sig);
            // store in local temp_db
            temp_db.push(TempUser {
                username,
                email,
                password,
            });
        }
        Err(err) => println!("Signup failed: {:?}", err),
    }
}

// ---------------- Signin ----------------
fn signin(
    client: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    program_id: &Pubkey,
    temp_db: &Vec<TempUser>,
) {
    let (username, password) = get_signin_details();
    let (pda, _bump) = Pubkey::find_program_address(&[username.as_bytes()], program_id);

    let ix = Instruction::new_with_bytes(
        *program_id,
        &UserInstruction::Signin {
            username: username.clone(),
            password: password.clone(),
        }
        .try_to_vec()
        .unwrap(),
        vec![AccountMeta::new(pda, false)],
    );

    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    tx.sign(&[payer], client.get_latest_blockhash().unwrap());

    match client.send_and_confirm_transaction(&tx) {
        Ok(sig) => {
            println!("Signin Tx sent: {}", sig);
            // also check from local temp_db
            if let Some(user) = temp_db.iter().find(|u| u.username == username) {
                if user.password == password {
                    println!("Local check: Welcome back, {}", user.username);
                } else {
                    println!("Local check: Wrong password");
                }
            } else {
                println!("Local check: User not found");
            }
        }
        Err(err) => println!("Signin failed: {:?}", err),
    }
}

// ---------------- List Users ----------------
fn list_users(temp_db: &Vec<TempUser>) {
    if temp_db.is_empty() {
        println!("No users signed up yet.");
    } else {
        println!("--- All Users ---");
        for (i, user) in temp_db.iter().enumerate() {
            println!("{}. {} ({})", i + 1, user.username, user.email);
        }
    }
}

// ---------------- functions  ----------------
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
