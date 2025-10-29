// use borsh::{BorshDeserialize, BorshSerialize};
// use dotenv::dotenv;
// use my_counter::{UserAccount, UserInstruction, UserRegistry};
// use solana_client::rpc_client::RpcClient;
// use solana_sdk::{
//     instruction::{AccountMeta, Instruction},
//     pubkey::Pubkey,
//     signature::{read_keypair_file, Signer},
//     system_program,
//     transaction::Transaction,
// };
// use spl_associated_token_account as ata;
// use std::{
//     env,
//     io::{self, Cursor, Write},
// };

// fn main() {
//     dotenv().ok();
//     let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");
//     let program_id: Pubkey = env::var("PROGRAM_ID")
//         .expect("PROGRAM_ID must be set")
//         .parse()
//         .unwrap();
//     let payer_path = env::var("PAYER_PATH").expect("PAYER_PATH must be set");

//     let client = RpcClient::new(rpc_url);
//     let payer = read_keypair_file(payer_path).unwrap();

//     loop {
//         println!("\n=== MENU ===\n");
//         println!("1. Signup");
//         println!("2. Signin");
//         println!("3. List Users");
//         println!("4. Create Currency");
//         println!("5. Create User Token Account");
//         println!("6. Mint To User");
//         println!("7. Transfer To User");
//         println!("8. Exit");
//         print!("Enter choice: ");
//         io::stdout().flush().unwrap();

//         let mut choice = String::new();
//         io::stdin().read_line(&mut choice).unwrap();

//         match choice.trim() {
//             "1" => signup(&client, &payer, &program_id),
//             "2" => signin(&client, &payer, &program_id),
//             "3" => list_users_onchain(&client, &program_id),
//             "4" => create_currency(&client, &payer, &program_id),
//             "5" => create_user_token_account(&client, &payer, &program_id),
//             "6" => mint_to_user(&client, &payer, &program_id),
//             // "7" => transfer_to_user(&client, &payer, &program_id),
//             "8" => {
//                 println!("Exiting...");
//                 break;
//             }
//             _ => println!("Invalid choice"),
//         }
//     }
// }

// // ---------------- SIGNUP ----------------
// fn signup(client: &RpcClient, payer: &solana_sdk::signer::keypair::Keypair, program_id: &Pubkey) {
//     let (username, email, password) = get_signup_details();
//     let (user_pda, _) = Pubkey::find_program_address(&[username.as_bytes()], program_id);
//     let (registry_pda, _) = Pubkey::find_program_address(&[b"registry"], program_id);

//     let ix = Instruction::new_with_bytes(
//         *program_id,
//         &UserInstruction::Signup {
//             username,
//             email,
//             password,
//         }
//         .try_to_vec()
//         .unwrap(),
//         vec![
//             AccountMeta::new(payer.pubkey(), true),
//             AccountMeta::new(user_pda, false),
//             AccountMeta::new(registry_pda, false),
//             AccountMeta::new_readonly(system_program::id(), false),
//         ],
//     );

//     send_tx(client, payer, &[ix]);
// }

// // ---------------- SIGNIN ----------------
// fn signin(client: &RpcClient, payer: &solana_sdk::signer::keypair::Keypair, program_id: &Pubkey) {
//     let (username, password) = get_signin_details();
//     let (user_pda, _) = Pubkey::find_program_address(&[username.as_bytes()], program_id);

//     let ix = Instruction::new_with_bytes(
//         *program_id,
//         &UserInstruction::Signin { username, password }
//             .try_to_vec()
//             .unwrap(),
//         vec![AccountMeta::new(user_pda, false)],
//     );

//     send_tx(client, payer, &[ix]);
// }

// // ---------------- LIST USERS ----------------
// fn list_users_onchain(client: &RpcClient, program_id: &Pubkey) {
//     let (registry_pda, _) = Pubkey::find_program_address(&[b"registry"], program_id);
//     println!("Fetching users from registry PDA: {}", registry_pda);

//     let account_data = match client.get_account_data(&registry_pda) {
//         Ok(data) => data,
//         Err(_) => {
//             println!("No registry found on-chain. No users yet.");
//             return;
//         }
//     };

//     let mut cursor = Cursor::new(&account_data);

//     let registry: UserRegistry = match UserRegistry::deserialize_reader(&mut cursor) {
//         Ok(r) => r,
//         Err(err) => {
//             println!("Failed to deserialize registry account: {:?}", err);
//             return;
//         }
//     };

//     if registry.users.is_empty() {
//         println!("No users registered yet.");
//         return;
//     }

//     println!("--- Registered Users ---");
//     for (index, user_pda) in registry.users.iter().enumerate() {
//         match client.get_account_data(user_pda) {
//             Ok(user_data) => match UserAccount::try_from_slice(&user_data) {
//                 Ok(user) => println!(
//                     "{}. Username: {}, Email: {}",
//                     index + 1,
//                     user.username,
//                     user.email
//                 ),
//                 Err(_) => println!("  Failed to deserialize user account {}", user_pda),
//             },
//             Err(_) => println!("  Could not fetch user account {}", user_pda),
//         }
//     }
// }

// // ---------------- CREATE CURRENCY ----------------
// fn create_currency(
//     client: &RpcClient,
//     payer: &solana_sdk::signer::keypair::Keypair,
//     program_id: &Pubkey,
// ) {
//     print!("Enter total supply (e.g., 100000): ");
//     io::stdout().flush().unwrap();
//     let mut input = String::new();
//     io::stdin().read_line(&mut input).unwrap();
//     let total_supply: u64 = input.trim().parse().unwrap_or(100000);

//     // generate new mint keypair
//     let mint = solana_sdk::signature::Keypair::new();
//     println!("Mint Pubkey: {}", mint.pubkey());

//     // derive MintData PDA
//     let (mint_data_pda, _) = Pubkey::find_program_address(&[mint.pubkey().as_ref()], program_id);

//     let token_program_pubkey = spl_token::id();
//     let rent_sysvar_pubkey = solana_sdk::sysvar::rent::id();

//     // Construct instruction to create currency
//     let ix = Instruction::new_with_bytes(
//         *program_id,
//         &UserInstruction::CreateCurrency { total_supply }
//             .try_to_vec()
//             .unwrap(),
//         vec![
//             AccountMeta::new(payer.pubkey(), true), // payer signs
//             AccountMeta::new(mint.pubkey(), true),  // mint must sign
//             AccountMeta::new(mint_data_pda, false), // mint data PDA store pannum total supply
//             AccountMeta::new_readonly(token_program_pubkey, false),
//             AccountMeta::new_readonly(rent_sysvar_pubkey, false),
//             AccountMeta::new_readonly(system_program::id(), false),
//         ],
//     );

//     // send transaction with both payer and mint as signers
//     send_tx_multi_signer(&client, &[&payer, &mint], &[ix]);

//     println!("Currency created with total supply {}", total_supply);
//     println!("MintData PDA: {}", mint_data_pda);
// }

// // ---------------- CREATE USER TOKEN ACCOUNT ----------------
// fn create_user_token_account(
//     client: &RpcClient,
//     payer: &solana_sdk::signer::keypair::Keypair,
//     program_id: &Pubkey,
// ) {
//     let username = get_input("Enter username: ");
//     let mint_pubkey_input = get_input("Enter Mint Pubkey: ");
//     let mint_pubkey: Pubkey = mint_pubkey_input.parse().expect("Invalid mint pubkey");

//     let (user_pda, _) = Pubkey::find_program_address(&[username.as_bytes()], program_id);
//     let token_account = ata::get_associated_token_address(&user_pda, &mint_pubkey);

//     println!("User token account will be: {}", token_account);

//     let ix = Instruction::new_with_bytes(
//         *program_id,
//         &UserInstruction::CreateUserTokenAccount {
//             username: username.clone(),
//             mint: mint_pubkey,
//         }
//         .try_to_vec()
//         .unwrap(),
//         vec![
//             AccountMeta::new(payer.pubkey(), true),
//             AccountMeta::new(user_pda, false),
//             AccountMeta::new(token_account, false),
//             AccountMeta::new_readonly(mint_pubkey, false),
//             AccountMeta::new_readonly(spl_token::id(), false),
//             AccountMeta::new_readonly(system_program::id(), false),
//             AccountMeta::new_readonly(solana_sdk::sysvar::rent::id(), false),
//             AccountMeta::new_readonly(ata::id(), false),
//         ],
//     );

//     send_tx(client, payer, &[ix]);
//     println!("User token account created for {}", username);
// }

// // ---------------- MINT TO USER ----------------
// fn mint_to_user(
//     client: &RpcClient,
//     payer: &solana_sdk::signer::keypair::Keypair,
//     program_id: &Pubkey,
// ) {
//     let username = get_input("Enter username: ");
//     let mint_pubkey_input = get_input("Enter Mint Pubkey: ");
//     let mint_pubkey: Pubkey = mint_pubkey_input.parse().expect("Invalid mint pubkey");
//     let amount_input = get_input("Enter amount to mint: ");
//     let amount: u64 = amount_input.parse().expect("Invalid amount");

//     let (user_pda, _) = Pubkey::find_program_address(&[username.as_bytes()], program_id);
//     let user_token_account = ata::get_associated_token_address(&user_pda, &mint_pubkey);

//     // derive MintData PDA
//     let (mint_data_pda, _) = Pubkey::find_program_address(&[mint_pubkey.as_ref()], program_id);

//     let ix = Instruction::new_with_bytes(
//         *program_id,
//         &UserInstruction::MintToUser {
//             username: username.clone(),
//             mint: mint_pubkey,
//             amount,
//         }
//         .try_to_vec()
//         .unwrap(),
//         vec![
//             AccountMeta::new(payer.pubkey(), true), //signs (true) — transaction payer & mint authority.
//             AccountMeta::new(user_pda, false),      //(PDA) read-only entry.
//             AccountMeta::new(user_token_account, false), // (ATA) where tokens will land.
//             AccountMeta::new(mint_pubkey, false),   //mint account.
//             AccountMeta::new(mint_data_pda, false), //supply tracking PDA.
//             AccountMeta::new_readonly(spl_token::id(), false), //read-only — token program reference.
//         ],
//     );

//     send_tx(client, payer, &[ix]);
//     println!("Attempted to mint {} tokens to {}", amount, username);
// }

// //----------------- TRANSFER TO USER -----------------
// // fn transfer_to_user(
// //     client: &RpcClient,
// //     payer: &solana_sdk::signer::keypair::Keypair,
// //     program_id: &Pubkey,
// // ) {
// //     let from_username = get_input("Enter sender username: ");
// //     let to_username = get_input("Enter a recipient username: ");
// //     let mint_pubkey_input = get_input("Enter Mint Pubkey: ");
// //     let mint_pubkey: Pubkey = mint_pubkey_input.parse().expect("Invalid mint pubkey");
// //     let amount_input = get_input("Enter amount to transfer: ");
// //     let amount: u64 = amount_input.parse().expect("Invalid amount");

// //     let (from_pda, _) = Pubkey::find_program_address(&[from_username.as_bytes()], program_id);
// //     let from_token_account = ata::get_associated_token_address(&from_pda, &mint_pubkey);
// //     let (to_pda, _) = Pubkey::find_program_address(&[to_username.as_bytes()], program_id);
// //     let to_token_account = ata::get_associated_token_address(&to_pda, &mint_pubkey);

// //     let ix = Instruction::new_with_bytes(
// //         *program_id,
// //         &UserInstruction::TransferToUser {
// //             from_username: from_username.clone(),
// //             to_username: to_username.clone(),
// //             mint: mint_pubkey,
// //             amount,
// //         }
// //         .try_to_vec()
// //         .unwrap(),
// //         vec![
// //             AccountMeta::new(from_pda, true), //  sender PDA (authority)
// //             AccountMeta::new(from_token_account, false),
// //             AccountMeta::new(to_pda, false),
// //             AccountMeta::new(to_token_account, false),
// //             AccountMeta::new_readonly(spl_token::id(), false),
// //         ],
// //     );

// //     send_tx(client, payer, &[ix]);

// //     println!(
// //         "Attempted to transfer {} tokens from {} to {}",
// //         amount, from_username, to_username
// //     );
// // }

// // ---------------- COMMON TX HELPERS ----------------
// fn send_tx(client: &RpcClient, payer: &solana_sdk::signer::keypair::Keypair, ix: &[Instruction]) {
//     let bh = client.get_latest_blockhash().unwrap();
//     let tx = Transaction::new_signed_with_payer(ix, Some(&payer.pubkey()), &[payer], bh);
//     match client.send_and_confirm_transaction(&tx) {
//         Ok(sig) => println!("Transaction sent successfully: {}", sig),
//         Err(e) => println!("Error sending transaction: {:?}", e),
//     }
// }

// fn send_tx_multi_signer(
//     client: &RpcClient,
//     signers: &[&solana_sdk::signer::keypair::Keypair],
//     ix: &[Instruction],
// ) {
//     let bh = client.get_latest_blockhash().unwrap();
//     let tx = Transaction::new_signed_with_payer(ix, Some(&signers[0].pubkey()), signers, bh);
//     match client.send_and_confirm_transaction(&tx) {
//         Ok(sig) => println!("Transaction sent successfully: {}", sig),
//         Err(e) => println!("Error sending transaction: {:?}", e),
//     }
// }

// // ---------------- INPUT HELPERS ----------------
// fn get_input(prompt: &str) -> String {
//     print!("{}", prompt);
//     io::stdout().flush().unwrap();
//     let mut input = String::new();
//     io::stdin().read_line(&mut input).unwrap();
//     input.trim().to_string()
// }

// fn get_signup_details() -> (String, String, String) {
//     let username = get_input("Username: ");
//     let email = get_input("Email: ");
//     let password = get_input("Password: ");
//     (username, email, password)
// }

// fn get_signin_details() -> (String, String) {
//     let username = get_input("Username: ");
//     let password = get_input("Password: ");
//     (username, password)
// }















// ======================================================================================
// --------------------------------------------------------------------------------------
// ======================================================================================

















// signup and signin new method

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
