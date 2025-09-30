use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(not(feature = "client"))]
use solana_program::entrypoint;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

// ======================
// Instructions
// ======================
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum UserInstruction {
    Signup {
        username: String,
        email: String,
        password: String,
    },
    Signin {
        username: String,
        password: String,
    },
}

// ======================
// User account structure
// ======================
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserAccount {
    pub username: String,
    pub email: String,
    pub password: String,
}

// ======================
// Entrypoint
// ======================
#[cfg(not(feature = "client"))]
entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("EntryPoint triggered...");

    let instruction = UserInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        UserInstruction::Signup {
            username,
            email,
            password,
        } => signup(accounts, username, email, password),
        UserInstruction::Signin { username, password } => signin(accounts, username, password),
    }
}

// ===========================
// Signup function
// ===========================
pub fn signup(
    accounts: &[AccountInfo],
    username: String,
    email: String,
    password: String,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;

    let mut user_data =
        UserAccount::try_from_slice(&user_account.data.borrow()).unwrap_or(UserAccount {
            username: "".to_string(),
            email: "".to_string(),
            password: "".to_string(),
        });

    user_data.username = username.clone();
    user_data.email = email.clone();
    user_data.password = password.clone();

    user_data.serialize(&mut &mut user_account.data.borrow_mut()[..])?;

    msg!(
        "Signup success! Stored username: {}, email: {}",
        username,
        email
    );
    Ok(())
}

// ===========================
// Signin function
// ===========================
pub fn signin(accounts: &[AccountInfo], username: String, password: String) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;

    let user_data = UserAccount::try_from_slice(&user_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    if user_data.username == username && user_data.password == password {
        msg!("Signin success! Welcome: {}", user_data.username);
        Ok(())
    } else {
        msg!("Signin failed: Invalid username or password");
        Err(ProgramError::InvalidArgument)
    }
}

// use borsh::{BorshDeserialize, BorshSerialize};
// //  BorshDeserialize -> Data Decode Panna / Read Panna use agum
// //  Borshserialize -> Data Encode Panna / Store Panna use agum
// #[cfg(not(feature = "client"))]
// use solana_program::entrypoint;
// use solana_program::{
//     account_info::{next_account_info, AccountInfo},
//     entrypoint::ProgramResult,
//     msg,
//     program_error::ProgramError,
//     pubkey::Pubkey,
// };

// // Instructions
// #[derive(BorshSerialize, BorshDeserialize, Debug)]
// pub enum UserInstruction {
//     Signup { username: String },
//     Signin { username: String },
// }

// // User account structure
// #[derive(BorshSerialize, BorshDeserialize, Debug)]
// pub struct UserAccount {
//     pub username: String,
//     pub email: String,
//     pub password: String,
// }

// // Entrypoint
// #[cfg(not(feature = "client"))] // ignore client side program
// entrypoint!(process_instruction); // starting point

// #[allow(dead_code)]
// // Indha code unused-a irukku
// fn process_instruction(
//     //function receive pannitu process pannuthu
//     _program_id: &Pubkey,
//     //program public key
//     accounts: &[AccountInfo],
//     //Instruction-ku attach panna list of accounts. (payer account, user PDA, system program.)
//     instruction_data: &[u8],
//     //Instruction payload / data (bytes).
// ) -> ProgramResult {
//     msg!("EntryPoint triggered...");

//     let instruction = UserInstruction::try_from_slice(instruction_data)
//         .map_err(|_| ProgramError::InvalidInstructionData)?;
//     //try_from_slice -> bytes -> UserInstruction struct convert pannum.
//     // |_| -> Any error vandhaalum, ignore the details Always convert to invalidinstructiondata

//     match instruction {
//         UserInstruction::Signup { username } => signup(accounts, username),
//         UserInstruction::Signin { username } => signin(username),
//     }
// }

// #[allow(dead_code)]
// //Indha code unused-a irukku

// fn signup(accounts: &[AccountInfo], username: String) -> ProgramResult {
//     let accounts_iter = &mut accounts.iter();
//     //accounts.iter() -> accounts array iterator create pannrom.

//     let user_account = next_account_info(accounts_iter)?;
//     //next_account_info -> iterator-la next account fetch panna utility function.

//     let mut user_data =
//         UserAccount::try_from_slice(&user_account.data.borrow()).unwrap_or(UserAccount {
//             username: "".to_string(),
//         });
//     //.borrow() -> Rust safety-ku immutable reference return pannum.
//     //Account la existing data try pannrom, illa na empty user create pannrom

//     user_data.username = username.clone();
//     //.clone() -> String copy, original preserve pannrom.
//     //Username update pannrom user_data struct la

//     user_data.serialize(&mut &mut user_account.data.borrow_mut()[..])?;
//     //mutable reference pass pannuthu so Borsh can write serialized bytes directly.
//     //Updated user_data -> bytes -> user_account storage la save pannrom.

//     msg!("Signup success, stored username: {}", username);
//     Ok(())
// }

// #[allow(dead_code)]
// fn signin(username: String) -> ProgramResult {
//     msg!("Signin attempt for username: {}", username);
//     Ok(())
// }

//-----------------------------------------------------------------------------------------------------------------
// Signin and Signup for explaination

// use borsh::{BorshDeserialize, BorshSerialize};
// // BorshDeserialize -> Data Decode Panna / Read Panna use agum
// // Borshserialize -> Data Encode Panna / Store Panna use agum
// #[cfg(not(feature = "client"))]
// use solana_program::entrypoint;
// use solana_program::{
//     account_info::{next_account_info, AccountInfo},
//     // AccountInfo -> Solana account metadata (balance,owner,lamports,data)
//     // next_account_info -> accounts iterator next account fetch panna
//     entrypoint::ProgramResult,
//     // Program Success -> ok(()) return panna
//     // Program Error -> Err(ProgramError) return panna
//     msg,
//     //log mathiri print agum
//     program::invoke_signed,
//     //invoke -> user already sign panirutha
//     //invoke-signed -> Pda sign panna vendiyathu
//     program_error::ProgramError,
//     //Solana default errors (InvaildAccountData,insufficientFunds,IncorrectProgramId)
//     pubkey::Pubkey,
//     //Solana public key type 32-byte unique identifier for account & programs
//     system_instruction,
//     //solana systemProgram instructions (create_account,transfer lamports,etc)
//     sysvar::{rent::Rent, Sysvar},
//     // Rent -> account la minimum balance hold panna
//     // Sysvar -> sysvar la irukura value fetch panna common API
// };

// #[derive(BorshDeserialize, BorshSerialize, Debug)]
// //If you don't add derive (or manually implements traits), you cannot use .tryfromslice() or trytovec()
// pub struct UserAccount {
//     pub username: String, // max 32 bytes
//     pub owner: Pubkey,    //32 bytes Public key of wallet address
//     pub bump: u8, //bump value account store panna , later some pda verification panna use agum.
// }

// #[derive(BorshDeserialize, BorshSerialize, Debug)]
// pub enum UserInstruction {
//     Signup { username: String }, //user wallet create pannitu username store panna vendiya data.
//     Signin,                      //pda verify panni, login operation perform panna
// }

// #[cfg(not(feature = "client"))] // ignore client side program
// entrypoint!(process_instruction); // starting point function

// pub fn process_instruction(
//     // accessible for another module
//     program_id: &Pubkey,      // reference for pubkey 32 byte
//     accounts: &[AccountInfo], //fields:(key,is_signer,is_writable,lamports,data) reference for multiple accounts username account and payer account
//     input: &[u8],             //wallet send panra instruction data.
// ) -> ProgramResult {
//     let instruction =
//         UserInstruction::try_from_slice(input).map_err(|_| ProgramError::InvalidInstructionData)?;
//     //try_from_slice -> Borsh deserialization function and we need to convert those bytes into meaningfull instruction
//     //|_|Any error vandhaalum, ignore the details Always convert to invalidinstructiondata
//     let accounts_iter = &mut accounts.iter();
//     //accounts array la irukura ellame iterrate pannuthu

//     match instruction {
//         //switch case mathiri
//         // check the signin or signup
//         UserInstruction::Signup { username } => {
//             msg!("Signup process");

//             let payer = next_account_info(accounts_iter)?;
//             //transcation send panra wallet accountinfo
//             let user_pda_info = next_account_info(accounts_iter)?;
//             //signup data store panna vendiya account
//             let system_program_info = next_account_info(accounts_iter)?;
//             //Pda account create panna, lamport,system_instruction call panna

//             if !payer.is_signer {
//                 //check panna, transcation send panra account signed must
//                 msg!("Payer must sign the transaction");
//                 return Err(ProgramError::MissingRequiredSignature);
//             }

//             let (pda, bump) =
//                 Pubkey::find_program_address(&[b"user", username.as_bytes()], &program_id);
//             //pda derive username and programid and send pda-transcation account key match
//             //PDA = program controlled account (no private key).
//             //Seeds = recipe to generate PDA.(sending sol, connecting to the solanablockchain,managing wallets and accounts)
//             //Bump = extra number to make sure PDA always valid. and signin for helper number
//             //find_program_address => generate a pda (program derivered address) pubkey + seeds
//             //return Pda public key and bump extra number vaild

//             if pda != *user_pda_info.key {
//                 // key means public key and * symbol is dereference for public key
//                 msg!("PDA mismatch");
//                 return Err(ProgramError::InvalidArgument);
//             }

//             // Pda data store agala naaa
//             if user_pda_info.data_is_empty() {
//                 let rent = Rent::get()?;
//                 //fetch current rent configuration

//                 let username_max_len = 32;
//                 // maximum length allowed for username

//                 let space = 4 + username_max_len + 32 + 1;
//                 //solana account create panna, allocate memory vendum
//                 //4 -> Borsh serialize panna, string first store length
//                 //max username bytes
//                 //32 -> maybe Pubkey storage (e.g., owner or another pubkey)
//                 //1 -> maybe bool (example: is_initialized)

//                 let lamports = rent.minimum_balance(space);
//                 //Calculate minimum lamports for PDA to be rent-exempt.

//                 let user_seeds: &[&[u8]] = &[b"user", username.as_bytes(), &[bump]];
//                 //PDA generate panna all seeds in one array

//                 let seeds = &[user_seeds]; // reference of user_seeds

//                 invoke_signed(
//                     //Normal accounts -> require private key //PDA -> no private key => program signs using seeds + bump
//                     &system_instruction::create_account(
//                         payer.key,
//                         //entha account create panna yaaru pays for lamports
//                         user_pda_info.key,
//                         //Must match PDA generated using seeds + bump
//                         lamports,
//                         space as u64,
//                         //Memory to allocate for the PDA
//                         program_id,
//                         //Owner of the new PDA account
//                     ),
//                     &[
//                         payer.clone(),
//                         //Must be signer
//                         user_pda_info.clone(),
//                         //PDA account to be created
//                         system_program_info.clone(),
//                         //System program account
//                     ],
//                     seeds,
//                 )?;
//             }

//             let user = UserAccount {
//                 username: username.clone(),
//                 //username copy in string
//                 owner: *payer.key,
//                 //dereference in actual wallet pubkey value
//                 bump: bump as u8,
//                 //PDA generate panna use panna bump value store pannuvom and PDA verification number
//             };
//             user.serialize(&mut &mut user_pda_info.data.borrow_mut()[..])?;
//             msg!("Signup successful for: {}", username);
//             Ok(())
//         }

//         UserInstruction::Signin => {
//             msg!("Signin process");

//             let signer = next_account_info(accounts_iter)?;
//             //namma user who signed transaction
//             let user_pda_info = next_account_info(accounts_iter)?;
//             //PDA account where user data will be stored

//             if !signer.is_signer {
//                 msg!("Signer required");
//                 return Err(ProgramError::MissingRequiredSignature);
//             }

//             let user = UserAccount::try_from_slice(&user_pda_info.data.borrow())
//                 .map_err(|_| ProgramError::InvalidAccountData)?;
//             //read PDA data -> convert to UserAccount struct

//             if user.owner != *signer.key {
//                 msg!("Signer does not match stored owner");
//                 return Err(ProgramError::InvalidAccountData);
//             }

//             msg!(
//                 "Signin success for user: {} (owner: {})",
//                 user.username,
//                 user.owner
//             );
//             Ok(())
//         }
//     }
// }

// TRANSCATION OF SIGNATURE AND METHOD

// use borsh::{BorshDeserialize, BorshSerialize};

// #[cfg(not(feature = "client"))]
// use solana_program::entrypoint;
// use solana_program::{
//     account_info::{next_account_info, AccountInfo},
//     entrypoint::ProgramResult,
//     msg,
//     pubkey::Pubkey,
// };

// #[derive(BorshSerialize, BorshDeserialize, Debug)]
// pub struct CounterAccount {
//     pub count: u32,
// }
// #[cfg(not(feature = "client"))]
// entrypoint!(process_instruction);

// pub fn process_instruction(
//     _program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     _instruction_data: &[u8],
// ) -> ProgramResult {
//     let accounts_iter = &mut accounts.iter();
//     let account = next_account_info(accounts_iter)?;

//     // Deserialize
//     let mut counter_data = CounterAccount::try_from_slice(&account.data.borrow()).(CounterAccount { count: 0 });

//     // Increment counter
//     counter_data.count += 1;
//     msg!("Counter increased: {}", counter_data.count);

//     // Serialize back
//     counter_data.serialize(&mut &mut account.data.borrow_mut()[..])?;

//     Ok(())
// }
