use borsh::{BorshDeserialize, BorshSerialize};
use my_counter::{UserAccount, UserInstruction};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Signer},
    transaction::Transaction,
};
use std::io::{self, Write};

fn main() {
    let rpc_url = "http://127.0.0.1:8899";
    let client = RpcClient::new(rpc_url.to_string());
    let program_id: Pubkey = "25SEofmnfm1QvKUiNtuEBrsfqXFf9WqaZinkCnspx1bg"
        .parse()
        .unwrap();
    let payer = read_keypair_file("/home/anjay-c/.config/solana/id.json").unwrap();

    loop {
        println!("\n1. Signup\n2. Signin\n3. Exit");
        print!("Enter choice: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim();

        match choice {
            "1" => {
                let (username, email, password) = get_signup_details();
                let (pda, _bump) =
                    Pubkey::find_program_address(&[username.as_bytes()], &program_id);

                let signup_ix = Instruction::new_with_bytes(
                    program_id,
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

                let mut tx = Transaction::new_with_payer(&[signup_ix], Some(&payer.pubkey()));
                tx.sign(&[&payer], client.get_latest_blockhash().unwrap());
                match client.send_and_confirm_transaction(&tx) {
                    Ok(sig) => println!("Signup success! Tx: {}", sig),
                    Err(e) => println!("Signup failed: {:?}", e),
                }
            }

            "2" => {
                let (username, password) = get_signin_details();
                let (pda, _bump) =
                    Pubkey::find_program_address(&[username.as_bytes()], &program_id);

                match client.get_account_data(&pda) {
                    Ok(data) => {
                        if let Ok(user_account) = UserAccount::try_from_slice(&data) {
                            if user_account.username == username
                                && user_account.password == password
                            {
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

            "3" => {
                println!("Exiting....");
                break;
            }

            _ => println!("Invalid choice!"),
        }
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
