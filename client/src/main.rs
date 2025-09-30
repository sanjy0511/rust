use borsh::BorshSerialize;
use my_counter::UserInstruction;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Signer},
    system_instruction,
    transaction::Transaction,
};
use std::io::{self, Write};
use std::str::FromStr;

fn main() {
    let rpc = RpcClient::new("http://127.0.0.1:8899".to_string());

    // Load wallet
    let payer = read_keypair_file(shellexpand::tilde("~/.config/solana/id.json").as_ref())
        .expect("Cannot read keypair");

    // Program ID
    let program_id = Pubkey::from_str("25SEofmnfm1QvKUiNtuEBrsfqXFf9WqaZinkCnspx1bg").unwrap();

    // PDA seed
    let user_seed = "user1"; // change if account already exists
    let user_pda = Pubkey::create_with_seed(&payer.pubkey(), user_seed, &program_id).unwrap();

    // Create PDA if it doesn't exist
    if rpc.get_account(&user_pda).is_err() {
        let lamports = rpc
            .get_minimum_balance_for_rent_exemption(std::mem::size_of::<my_counter::UserAccount>())
            .unwrap();

        let create_ix = system_instruction::create_account_with_seed(
            &payer.pubkey(),
            &user_pda,
            &payer.pubkey(),
            user_seed,
            lamports,
            std::mem::size_of::<my_counter::UserAccount>() as u64,
            &program_id, // owner must be program
        );

        let mut tx = Transaction::new_with_payer(&[create_ix], Some(&payer.pubkey()));
        tx.sign(&[&payer], rpc.get_latest_blockhash().unwrap());

        match rpc.send_and_confirm_transaction(&tx) {
            Ok(sig) => println!("PDA account created: {}", sig),
            Err(e) => {
                eprintln!("Failed to create PDA account: {:?}", e);
                return;
            }
        }
    } else {
        println!("PDA account already exists, skipping creation");
    }

    // Manual input
    let mut username = String::new();
    let mut email = String::new();
    let mut password = String::new();

    print!("Enter username: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut username).unwrap();
    let username = username.trim().to_string();

    print!("Enter email: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut email).unwrap();
    let email = email.trim().to_string();

    print!("Enter password: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut password).unwrap();
    let password = password.trim().to_string();

    // ======================
    // Signup transaction
    // ======================
    let signup_ix = Instruction::new_with_bytes(
        program_id,
        &UserInstruction::Signup {
            username: username.clone(),
            email: email.clone(),
            password: password.clone(),
        }
        .try_to_vec()
        .unwrap(),
        vec![AccountMeta::new(user_pda, false)],
    );

    let mut signup_tx = Transaction::new_with_payer(&[signup_ix], Some(&payer.pubkey()));
    signup_tx.sign(&[&payer], rpc.get_latest_blockhash().unwrap());

    match rpc.send_and_confirm_transaction(&signup_tx) {
        Ok(sig) => println!("Signup transaction sent: {}", sig),
        Err(e) => eprintln!("Signup failed: {:?}", e),
    }

    // ======================
    // Signin transaction
    // ======================
    let signin_ix = Instruction::new_with_bytes(
        program_id,
        &UserInstruction::Signin {
            username: username.clone(),
            password: password.clone(),
        }
        .try_to_vec()
        .unwrap(),
        vec![AccountMeta::new(user_pda, false)],
    );

    let mut signin_tx = Transaction::new_with_payer(&[signin_ix], Some(&payer.pubkey()));
    signin_tx.sign(&[&payer], rpc.get_latest_blockhash().unwrap());

    match rpc.send_and_confirm_transaction(&signin_tx) {
        Ok(sig) => println!("Signin transaction sent: {}", sig),
        Err(e) => eprintln!("Signin failed: {:?}", e),
    }
}

// use borsh::BorshSerialize;
// // Rust Struct/Enum its convert to bytes
// use solana_client::rpc_client::RpcClient;
// //solana Network (devnet/testnet/mainnet) comminicate panni. Balance check, transcation send,account info edukka use pannrom
// use solana_sdk::{
//     instruction::{AccountMeta, Instruction},
//     //AccountMeta -> Solana accountinfo (writable,signer) define panna
//     //instruction -> solana program oru single instruction
//     pubkey::Pubkey,
//     //solana account/program address (32 bytes)
//     signature::{read_keypair_file, Signer},
//     //read_keypair_file -> localfile irukkura private key read pannum
//     //signer -> Transcation sign panna mudiyum (wallet)
//     system_instruction,
//     //default solana operation (accountcreate,transfer)
//     transaction::Transaction,
//     //Multiple instruction bundle, Blockchain-ku send pannunum
// };
// use std::str::FromStr;
// //Pubkey objectahh convert pannuvom

// use my_counter::UserInstruction;
// //program define panna custom instruction enum. SIGNIN OR SIGNUP

// fn main() {
//     // Program starting point
//     let rpc = RpcClient::new("http://127.0.0.1:8899".to_string());
//     //client connection open pannuthu, local solana validator oda

//     let payer = read_keypair_file(shellexpand::tilde("~/.config/solana/id.json").as_ref())
//         .expect("Cannot read keypair");
//     //read_keypair_file -> wallet oda private key file read pannuthu
//     //path -> default solana Cli wallet store place
//     //shellexpand::tilde -> full path ahh expand pannuthu
//     //cow<str>.as_ref() -> Box la irukura string ah eduthu, normal string-a convert pannuthu
//     //expect -> wrong na program crash aagum with error msg.
//     //payer -> wallet entha account dhan transcation fees pay pannum.

//     let program_id = Pubkey::from_str("25SEofmnfm1QvKUiNtuEBrsfqXFf9WqaZinkCnspx1bg").unwrap();
//     //solana program address
//     //from_str -> String ah Pubkey object-ah convert pannuthu
//     //unwrap() -> error handle pannuthu

//     let user_seed = "user1";
//     //Seeds = recipe to generate PDA.(sending sol, connecting to the solanablockchain,managing wallets and accounts)

//     let user_pda = Pubkey::create_with_seed(&payer.pubkey(), user_seed, &program_id).unwrap();
//     //create_with_seed -> pda account create panna
//     //&payer.pubkey() -> base account (main wallet)
//     //&program_id -> Target program id (ithu account link aagum)
//     //unwrap -> program crash agamaa, error handle pannum

//     if rpc.get_account(&user_pda).is_err() {
//         //PDA account already exist check pannu
//         //is_err -> account illaina true return pannum

//         let lamports = rpc.get_minimum_balance_for_rent_exemption(5000).unwrap();
//         // get_minimum_balance_for_rent_exemption -> 5000 bytes data-ku rent-free minimum lamports calculate pannum.
//         // bigger space kuduthuruken multiple users store panna

//         let create_ix = system_instruction::create_account_with_seed(
//             &payer.pubkey(), // -> Fee pay panna wallet
//             &user_pda,       // -> New account address (PDA)
//             &payer.pubkey(), // -> Base account for seed
//             user_seed,       // -> Seed string
//             lamports,        // -> Account-ku fund amount (rent-exempt)
//             5000,            // -> Space in bytes (multiple users store panna enough size)
//             &program_id,     // -> Which program control pannum
//         );

//         let mut tx = Transaction::new_with_payer(&[create_ix], Some(&payer.pubkey()));
//         //create_ix -> Account create panna instruction.
//         //[create_ix] -> Transaction-ku instructions list.
//         //Some(&payer.pubkey()) -> Transaction fee pay panna wallet.

//         tx.sign(&[&payer], rpc.get_latest_blockhash().unwrap());
//         // &[&payer] -> Yaar transaction sign pannuvano (our wallet).
//         // rpc.get_latest_blockhash() -> Latest blockchain hash eduthu, transaction valid-a irukka check pannum.

//         match rpc.send_and_confirm_transaction(&tx) {
//             //send_and_confirm_transaction -> Transaction blockchain-ku send pannuthu and wait until confirmed.
//             Ok(sig) => println!("PDA account created: {}", sig),
//             Err(e) => {
//                 eprintln!("Failed to create PDA account: {:?}", e);
//                 return;
//             }
//         }
//     } else {
//         println!("PDA account already exists, skipping creation");
//     }

//     // Signup
//     let signup_ix = Instruction::new_with_bytes(
//         program_id, // program_id ku instruction send pannuthu
//         &UserInstruction::Signup {
//             username: "Sanjay".to_string(),
//             email: "sanjay@test.com".to_string(),
//             password: "12345".to_string(),
//             // signup instruction create pannitu username + email + password set pannuthu
//         }
//         .try_to_vec()
//         //indha data (Signup info) -> bytes a serlize pannuthu
//         .unwrap(),
//         //error handling
//         vec![AccountMeta::new(user_pda, false)],
//         //vec! -> Instruction-ku attach panna accounts list create pannudhu.
//         //AccountMeta -> Solana instruction-ku account info define panna structure.
//     );

//     let mut tx = Transaction::new_with_payer(&[signup_ix], Some(&payer.pubkey()));
//     //Transaction::new_with_payer -> Transaction na blockchain-ku send panna set of instructions.
//     // &[signup_ix] -> array list of reference.
//     // Some(&payer.pubkey()) -> Optional value ahh edukuthu

//     tx.sign(&[&payer], rpc.get_latest_blockhash().unwrap());
//     //Proof kudukka -> intha transaction payer dhaan send panran
//     //Blockchain accept panna signature must.
//     //Blockhash => recent blockchain block-oda unique fingerprint (hash).

//     match rpc.send_and_confirm_transaction(&tx) {
//         //send_and_confirm_transaction -> Transaction blockchain-ku send pannuthu and wait until confirmed.
//         Ok(sig) => println!("Signup transaction sent: {}", sig),
//         Err(e) => eprintln!("Signup failed: {:?}", e),
//     }

//     // Signin
//     let signin_ix = Instruction::new_with_bytes(
//         program_id,
//         &UserInstruction::Signin {
//             username: "Sanjay".to_string(),
//             password: "12345".to_string(),
//             // signin instruction create pannitu username + password check panna
//         }
//         .try_to_vec()
//         .unwrap(),
//         vec![AccountMeta::new(user_pda, false)],
//     );

//     let mut tx2 = Transaction::new_with_payer(&[signin_ix], Some(&payer.pubkey()));
//     //Transaction::new_with_payer -> Transaction na blockchain-ku send panna set of instructions.
//     // &[signin_ix] -> array list of reference.
//     // Some(&payer.pubkey()) -> Optional value ahh edukuthu

//     tx2.sign(&[&payer], rpc.get_latest_blockhash().unwrap());
//     //send_and_confirm_transaction -> Transaction blockchain-ku send pannuthu and wait until confirmed.

//     match rpc.send_and_confirm_transaction(&tx2) {
//         Ok(sig) => println!("Signin transaction sent: {}", sig),
//         Err(e) => eprintln!("Signin failed: {:?}", e),
//     }
// }

//..................................................................................................................
// // Signin and Signup

// use borsh::{BorshDeserialize, BorshSerialize};
// use shellexpand;
// use solana_client::rpc_client::RpcClient;
// use solana_sdk::instruction::{AccountMeta, Instruction};
// use solana_sdk::pubkey::Pubkey;
// use solana_sdk::signature::{read_keypair_file, Signer};
// use solana_sdk::transaction::Transaction;
// use std::str::FromStr;

// #[derive(BorshSerialize, BorshDeserialize, Debug)]
// pub enum UserInstruction {
//     Signup { username: String },
//     Signin,
// }

// fn main() {
//     let rpc_url = "http://127.0.0.1:8899";
//     let client = RpcClient::new(rpc_url.to_string());

//     // Load payer
//     let payer_path = shellexpand::tilde("~/.config/solana/id.json");
//     let payer = match read_keypair_file(payer_path.as_ref()) {
//         Ok(kp) => kp,
//         Err(e) => {
//             eprintln!("Failed to read keypair: {}", e);
//             return;
//         }
//     };

//     // Program ID
//     let program_id = match Pubkey::from_str("G3uVF3obBogx1QEVPsBfZ6DfiaRGtsAP1sTLCkYfHiZH") {
//         Ok(pk) => pk,
//         Err(e) => {
//             eprintln!("Invalid program ID: {}", e);
//             return;
//         }
//     };

//     let username = "alice".to_string();
//     let (user_pda, bump) =
//         Pubkey::find_program_address(&[b"user", username.as_bytes()], &program_id);
//     println!("User PDA: {} bump {}", user_pda, bump);

//     // ---------------- Signup ----------------
//     let signup_data = UserInstruction::Signup {
//         username: username.clone(),
//     }
//     .try_to_vec()
//     .unwrap();

//     let signup_ix = Instruction::new_with_bytes(
//         program_id,
//         &signup_data,
//         vec![
//             AccountMeta::new(payer.pubkey(), true),
//             AccountMeta::new(user_pda, false),
//             AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
//         ],
//     );

//     let recent_blockhash = match client.get_latest_blockhash() {
//         Ok(hash) => hash,
//         Err(e) => {
//             eprintln!("Failed to get blockhash: {}", e);
//             return;
//         }
//     };

//     let tx = Transaction::new_signed_with_payer(
//         &[signup_ix],
//         Some(&payer.pubkey()),
//         &[&payer],
//         recent_blockhash,
//     );

//     match client.send_and_confirm_transaction(&tx) {
//         Ok(sig) => println!("Signup tx: {}", sig),
//         Err(e) => eprintln!("Signup transaction failed: {}", e),
//     }

//     // ---------------- Signin ----------------
//     let signin_data = UserInstruction::Signin.try_to_vec().unwrap();
//     let signin_ix = Instruction::new_with_bytes(
//         program_id,
//         &signin_data,
//         vec![
//             AccountMeta::new(payer.pubkey(), true),
//             AccountMeta::new(user_pda, false),
//             AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
//         ],
//     );

//     let recent_blockhash = match client.get_latest_blockhash() {
//         Ok(hash) => hash,
//         Err(e) => {
//             eprintln!("Failed to get blockhash: {}", e);
//             return;
//         }
//     };

//     let tx2 = Transaction::new_signed_with_payer(
//         &[signin_ix],
//         Some(&payer.pubkey()),
//         &[&payer],
//         recent_blockhash,
//     );

//     match client.send_and_confirm_transaction(&tx2) {
//         Ok(sig) => println!("Signin tx: {}", sig),
//         Err(e) => eprintln!("Signin transaction failed: {}", e),
//     }
// }

// use anchor_client::solana_sdk::signature::{read_keypair_file, Keypair};
// use anchor_client::{Client, Cluster};
// use solana_sdk::pubkey::Pubkey;

// fn main() {
//     let payer = read_keypair_file("~/.config/solana/id.json").unwrap();
//     let client = Client::new_with_options(Cluster::Devnet, payer.clone(), Default::default());

//     let program_id = Pubkey::from_str("YOUR_PROGRAM_ID_HERE").unwrap();
//     let program = client.program(program_id);

//     // Signup
//     let user_seed = b"alice";
//     let user_pda = Pubkey::find_program_address(&[user_seed], &program_id).0;

//     program
//         .request()
//         .accounts(solana_auth::accounts::Signup {
//             user: user_pda,
//             user_wallet: payer.pubkey(),
//             system_program: solana_sdk::system_program::id(),
//         })
//         .args(solana_auth::instruction::Signup {
//             username: "alice".to_string(),
//         })
//         .send()
//         .unwrap();

//     println!(" Signup successful");

//     // Signin
//     let user_account: solana_auth::UserAccount = program.account(user_pda).unwrap();
//     if user_account.is_registered {
//         println!("Signin successful for {}", user_account.username);
//     } else {
//         println!(" User not registered");
//     }
// }

// TRANSCATION SIGNATURE IN SOLANA METHOD DEPLOY

// use borsh::BorshDeserialize;
// use my_counter::CounterAccount;
// use shellexpand;
// use solana_client::rpc_client::RpcClient;
// use solana_sdk::{
//     instruction::{AccountMeta, Instruction},
//     pubkey::Pubkey,
//     signature::{read_keypair_file, Keypair, Signer},
//     system_instruction,
//     transaction::Transaction,
// };

// fn main() {
//     // Connect to Devnet
//     let rpc = RpcClient::new("https://api.devnet.solana.com".to_string());
//     // let rpc = RpcClient::new("http://127.0.0.1:8899".to_string());

//     // Load payer wallet
//     let payer = read_keypair_file(shellexpand::tilde("~/.config/solana/id.json").to_string())
//         .expect("Cannot read keypair");

//     // Generate counter account keypair
//     let counter = Keypair::new();

//     // Rent exempt balance
//     let space = std::mem::size_of::<CounterAccount>();
//     let lamports = rpc.get_minimum_balance_for_rent_exemption(space).unwrap();

//     // Deployed Program ID
//     let program_id: Pubkey = "HejpzH4qkTQ9mKVvyNkttptfrMR7DRLVBnGsoMXdCSU5"
//         .parse()
//         .unwrap();

//     // Instruction 1: create counter account
//     let create_account_ix = system_instruction::create_account(
//         &payer.pubkey(),
//         &counter.pubkey(),
//         lamports,
//         space as u64,
//         &program_id,
//     );

//     // Instruction 2: call your program
//     let program_ix = Instruction::new_with_bincode::<Vec<u8>>(
//         program_id,
//         &vec![], // empty instruction data
//         vec![AccountMeta::new(counter.pubkey(), false)],
//     );

//     // Build transaction
//     let blockhash = rpc.get_latest_blockhash().unwrap();
//     let tx = Transaction::new_signed_with_payer(
//         &[create_account_ix, program_ix],
//         Some(&payer.pubkey()),
//         &[&payer, &counter],
//         blockhash,
//     );

//     // Send transaction
//     let sig = rpc.send_and_confirm_transaction(&tx).unwrap();
//     println!("Transaction Signature: {}", sig);

//     // Fetch counter value
//     let acc = rpc.get_account(&counter.pubkey()).unwrap();
//     let counter_state: CounterAccount = CounterAccount::try_from_slice(&acc.data).unwrap();
//     println!("Counter value = {}", counter_state.count);
// }
