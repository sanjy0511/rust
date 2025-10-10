use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(not(feature = "client"))]
use solana_program::entrypoint;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use spl_associated_token_account::instruction as ata_instruction;
use spl_token::instruction as token_instruction;
use std::io::Cursor;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct UserAccount {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default, Clone)]
pub struct UserRegistry {
    pub users: Vec<Pubkey>,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct MintData {
    pub total_supply: u64,
    pub current_supply: u64,
}

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
    ListUsers,
    CreateCurrency {
        total_supply: u64,
    },
    CreateUserTokenAccount {
        username: String,
        mint: Pubkey,
    },
    MintToUser {
        username: String,
        mint: Pubkey,
        amount: u64,
    },
    TransferToUser {
        from_username: String,
        to_username: String,
        mint: Pubkey,
        amount: u64,
    },
}

#[cfg(not(feature = "client"))]
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Entrypoint Triggered");

    let instruction = UserInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        UserInstruction::Signup {
            username,
            email,
            password,
        } => signup(program_id, accounts, username, email, password),
        UserInstruction::Signin { username, password } => signin(accounts, username, password),
        UserInstruction::ListUsers => list_users(accounts),
        UserInstruction::CreateCurrency { total_supply } => {
            create_currency(program_id, accounts, total_supply)
        }
        UserInstruction::CreateUserTokenAccount { username, mint } => {
            create_user_token_account(program_id, accounts, username, mint)
        }
        UserInstruction::MintToUser {
            username,
            mint,
            amount,
        } => mint_to_user(program_id, accounts, username, mint, amount),
        UserInstruction::TransferToUser {
            from_username,
            to_username,
            mint,
            amount,
        } => transfer_to_user(
            program_id,
            accounts,
            from_username,
            to_username,
            mint,
            amount,
        ),
    }
}

// ---------------- USER FUNCTIONS ----------------
pub fn signup(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    username: String,
    email: String,
    password: String,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let user_account = next_account_info(accounts_iter)?;
    let registry_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    let user_data = UserAccount {
        username: username.clone(),
        email,
        password,
    };
    let (user_pda, user_bump) = Pubkey::find_program_address(&[username.as_bytes()], program_id);

    if user_account.key != &user_pda {
        msg!("Invalid PDA provided for user");
        return Err(ProgramError::InvalidArgument);
    }

    let user_size = user_data.try_to_vec()?.len();
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(user_size);

    if user_account.lamports() == 0 {
        msg!("Creating user PDA...");
        let create_user_ix = system_instruction::create_account(
            payer.key,
            &user_pda,
            lamports,
            user_size as u64,
            program_id,
        );
        invoke_signed(
            &create_user_ix,
            &[payer.clone(), user_account.clone(), system_program.clone()],
            &[&[username.as_bytes(), &[user_bump]]],
        )?;
    }

    user_account.data.borrow_mut().fill(0);
    user_data.serialize(&mut &mut user_account.data.borrow_mut()[..])?;
    msg!("User saved successfully!");

    // Registry PDA
    let (registry_pda, registry_bump) = Pubkey::find_program_address(&[b"registry"], program_id);
    if registry_account.key != &registry_pda {
        msg!("Invalid registry PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let registry_space = 8192usize;
    let registry_lamports = rent.minimum_balance(registry_space);

    if registry_account.lamports() == 0 {
        msg!("Creating new registry PDA...");
        let create_registry_ix = system_instruction::create_account(
            payer.key,
            &registry_pda,
            registry_lamports,
            registry_space as u64,
            program_id,
        );
        invoke_signed(
            &create_registry_ix,
            &[
                payer.clone(),
                registry_account.clone(),
                system_program.clone(),
            ],
            &[&[b"registry", &[registry_bump]]],
        )?;

        let empty = UserRegistry::default();
        empty.serialize(&mut &mut registry_account.data.borrow_mut()[..])?;
        msg!("Empty registry initialized!");
    }

    let registry_data_copy: Vec<u8> = registry_account.data.borrow().to_vec();
    let mut registry = match UserRegistry::deserialize_reader(&mut Cursor::new(&registry_data_copy))
    {
        Ok(r) => r,
        Err(_) => UserRegistry::default(),
    };

    if !registry.users.contains(&user_pda) {
        registry.users.push(user_pda);
    }

    let serialized = registry.try_to_vec()?;
    let mut data_mut = registry_account.data.borrow_mut();
    data_mut[..serialized.len()].copy_from_slice(&serialized);

    msg!(
        "Registry updated successfully with {} users",
        registry.users.len()
    );
    Ok(())
}

pub fn signin(accounts: &[AccountInfo], username: String, password: String) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;

    let stored_user = UserAccount::try_from_slice(&user_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    if stored_user.username == username && stored_user.password == password {
        msg!("Signin success! Welcome {}", stored_user.username);
        Ok(())
    } else {
        msg!("Signin failed: Invalid credentials");
        Err(ProgramError::InvalidAccountData)
    }
}

pub fn list_users(accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let registry_account = next_account_info(accounts_iter)?;
    let registry_data_copy: Vec<u8> = registry_account.data.borrow().to_vec();

    if registry_data_copy.is_empty() {
        msg!("Registry account empty!");
        return Ok(());
    }

    let registry: UserRegistry =
        match UserRegistry::deserialize_reader(&mut Cursor::new(&registry_data_copy)) {
            Ok(r) => r,
            Err(e) => {
                msg!("Failed to deserialize registry: {:?}", e);
                return Ok(());
            }
        };

    if registry.users.is_empty() {
        msg!("No users registered yet.");
        return Ok(());
    }

    msg!("-------- Registered Users --------");
    for (i, user_pda) in registry.users.iter().enumerate() {
        msg!("{}. User PDA: {}", i + 1, user_pda);
    }
    Ok(())
}

// ---------------- TOKEN FUNCTIONS ----------------
pub fn create_currency(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    total_supply: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let mint_account = next_account_info(accounts_iter)?;
    let mint_data_account = next_account_info(accounts_iter)?; // PDA to store MintData
    let token_program = next_account_info(accounts_iter)?;
    let rent_sysvar = next_account_info(accounts_iter)?;

    let rent = Rent::get()?;
    let mint_size = spl_token::state::Mint::LEN;
    let lamports = rent.minimum_balance(mint_size);

    msg!("Creating Mint account...");
    invoke(
        //Token program ah owner-a assign pannum.
        &system_instruction::create_account(
            payer.key,
            mint_account.key,
            lamports,
            mint_size as u64,
            token_program.key,
        ),
        &[payer.clone(), mint_account.clone()],
    )?;

    msg!("Initializing Mint...");
    invoke(
        &token_instruction::initialize_mint(
            token_program.key, // which token program to use
            mint_account.key,  // which mint account to initialize
            payer.key,         // mint authority (who can mint new tokens)
            None,              // freeze authority (optional; None = no freeze authority)
            0,                 // decimal places (0 = no fractional tokens)
        )?,
        &[
            mint_account.clone(),
            rent_sysvar.clone(),
            token_program.clone(),
        ],
    )?;

    // MintData PDA using program_id
    let (mint_data_pda, mint_data_bump) =
        Pubkey::find_program_address(&[mint_account.key.as_ref()], program_id);

    if mint_data_account.key != &mint_data_pda {
        msg!("Invalid MintData PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let mint_data = MintData {
        total_supply,
        current_supply: 0,
    };
    let size = mint_data.try_to_vec()?.len();
    let lamports_mintdata = rent.minimum_balance(size);

    if mint_data_account.lamports() == 0 {
        msg!("Creating MintData PDA...");
        let ix = system_instruction::create_account(
            payer.key,
            &mint_data_pda,
            lamports_mintdata,
            size as u64,
            program_id,
        );
        invoke_signed(
            &ix,
            &[payer.clone(), mint_data_account.clone()],
            &[&[mint_account.key.as_ref(), &[mint_data_bump]]],
        )?;
    }

    mint_data.serialize(&mut &mut mint_data_account.data.borrow_mut()[..])?;
    msg!("Currency created with total supply {}", total_supply);
    Ok(())
}

pub fn create_user_token_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    username: String,
    mint: Pubkey,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let user_account = next_account_info(accounts_iter)?; //PDA derived from username
    let token_account = next_account_info(accounts_iter)?; //user’s associated token account (ATA)
    let mint_account = next_account_info(accounts_iter)?; //which token mint this account belongs to
    let token_program = next_account_info(accounts_iter)?; //To handle token logic (mint, transfer, ATA, etc.)
    let system_program = next_account_info(accounts_iter)?; //To create accounts / fund lamports
    let rent_sysvar = next_account_info(accounts_iter)?; //to check rent exemption balance
    let ata_program = next_account_info(accounts_iter)?; //To auto-create token accounts for users

    let (user_pda, _) = Pubkey::find_program_address(&[username.as_bytes()], program_id);

    if user_account.key != &user_pda {
        msg!("Invalid User PDA");
        return Err(ProgramError::InvalidArgument);
    }

    msg!("Creating associated token account for user {}", username);

    invoke(
        &ata_instruction::create_associated_token_account(
            payer.key,
            &user_pda,
            &mint,
            token_program.key,
        ),
        &[
            payer.clone(),
            token_account.clone(),
            user_account.clone(),
            mint_account.clone(),
            system_program.clone(),
            token_program.clone(),
            rent_sysvar.clone(),
            ata_program.clone(),
            //Instruction-ku required accounts list.
        ],
    )?;

    msg!("User Token Account created successfully!");
    Ok(())
}

pub fn mint_to_user(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    username: String,
    _mint: Pubkey,
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?; //(transaction signer).
    let user_account = next_account_info(accounts_iter)?; //should be the User PDA
    let user_token_account = next_account_info(accounts_iter)?; //the user's ATA — token account to credit.
    let mint_account = next_account_info(accounts_iter)?; //the SPL mint account (the token definition).
    let mint_data_account = next_account_info(accounts_iter)?; //the MintData PDA account where you stored total_supply and current_supply
    let token_program = next_account_info(accounts_iter)?; // SPL Token Program account (for CPI).

    let (user_pda, _bump) = Pubkey::find_program_address(&[username.as_bytes()], program_id);
    if user_account.key != &user_pda {
        msg!("Invalid user PDA");
        return Err(ProgramError::InvalidArgument);
    }

    // Deserialize mint data
    let mut mint_data: MintData = MintData::try_from_slice(&mint_data_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Check supply limit
    if mint_data.current_supply + amount > mint_data.total_supply {
        msg!(
            "Cannot mint {} tokens. Exceeds total supply of {}",
            amount,
            mint_data.total_supply
        );
        return Err(ProgramError::Custom(0));
    }

    msg!("Minting {} tokens to {}", amount, user_token_account.key);

    let ix = token_instruction::mint_to(
        token_program.key,
        mint_account.key,
        user_token_account.key,
        payer.key,
        &[],
        amount,
    )?;

    invoke(
        &ix,
        &[
            mint_account.clone(),
            user_token_account.clone(),
            payer.clone(),
            token_program.clone(),
        ],
    )?;

    mint_data.current_supply += amount;
    mint_data.serialize(&mut &mut mint_data_account.data.borrow_mut()[..])?;

    msg!(
        "Tokens minted successfully! Current supply: {}",
        mint_data.current_supply
    );
    Ok(())
}

// pub fn transfer_to_user(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     from_username: String,
//     to_username: String,
//     _mint: Pubkey,
//     amount: u64,
// ) -> ProgramResult {
//     let accounts_iter = &mut accounts.iter();
//     let sender_account = next_account_info(accounts_iter)?; // PDA of sender (authority)
//     let sender_token_account = next_account_info(accounts_iter)?; // sender's ATA
//     let recipient_account = next_account_info(accounts_iter)?; // PDA of receiver
//     let recipient_token_account = next_account_info(accounts_iter)?; // receiver's ATA
//     let token_program = next_account_info(accounts_iter)?; // SPL token program

//     let (sender_pda, sender_bump) =
//         Pubkey::find_program_address(&[from_username.as_bytes()], program_id);
//     let (recipient_pda, _recipient_bump) =
//         Pubkey::find_program_address(&[to_username.as_bytes()], program_id);

//     if sender_account.key != &sender_pda || recipient_account.key != &recipient_pda {
//         msg!("Invalid PDA for sender or recipient");
//         return Err(ProgramError::InvalidArgument);
//     }

//     msg!(
//         "Transferring {} tokens from {} to {}",
//         amount,
//         from_username,
//         to_username
//     );

//     // Construct the transfer instruction
//     let ix = token_instruction::transfer(
//         token_program.key,
//         sender_token_account.key,
//         recipient_token_account.key,
//         sender_account.key, // authority = PDA
//         &[],
//         amount,
//     )?;

//     // Use invoke_signed so PDA can sign using seeds
//     invoke_signed(
//         &ix,
//         &[
//             sender_token_account.clone(),
//             recipient_token_account.clone(),
//             sender_account.clone(),
//             token_program.clone(),
//         ],
//         &[&[from_username.as_bytes(), &[sender_bump]]], // PDA seeds for signing
//     )?;

//     msg!("Transfer successful!");
//     Ok(())
// }
