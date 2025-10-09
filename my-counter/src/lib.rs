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
    },
    MintToUser {
        username: String,
        amount: u64,
    },
    TransferToUser {
        from_username: String,
        to_username: String,
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
        UserInstruction::CreateCurrency { total_supply } => create_currency(accounts, total_supply),
        UserInstruction::CreateUserTokenAccount { username } => {
            create_user_token_account(program_id, accounts, username)
        }
        UserInstruction::MintToUser { username, amount } => {
            mint_to_user(program_id, accounts, username, amount)
        }
        UserInstruction::TransferToUser {
            from_username,
            to_username,
            amount,
        } => transfer_to_user(program_id, accounts, from_username, to_username, amount),
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
pub fn create_currency(accounts: &[AccountInfo], total_supply: u64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?;
    let mint_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let rent_sysvar = next_account_info(accounts_iter)?;

    let rent = Rent::get()?;
    let mint_size = spl_token::state::Mint::LEN;
    let lamports = rent.minimum_balance(mint_size);

    msg!("Creating Mint Token...!");
    invoke(
        &system_instruction::create_account(
            payer.key,
            mint_account.key,
            lamports,
            mint_size as u64,
            token_program.key,
        ),
        &[payer.clone(), mint_account.clone()],
    )?;

    msg!("Initializing Mint...!");
    invoke(
        &token_instruction::initialize_mint(
            token_program.key,
            mint_account.key,
            payer.key,
            None,
            0,
        )?,
        &[
            mint_account.clone(),
            rent_sysvar.clone(),
            token_program.clone(),
        ],
    )?;

    msg!("Currency created with total supply {}", total_supply);
    Ok(())
}

pub fn create_user_token_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    username: String,
) -> ProgramResult {
    msg!("=== On-chain Debug: CreateUserTokenAccount ===");

    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?; // payer (signer)
    let user_token_account = next_account_info(accounts_iter)?; // PDA
    let mint_account = next_account_info(accounts_iter)?; // token mint
    let token_program = next_account_info(accounts_iter)?; // SPL token program
    let system_program = next_account_info(accounts_iter)?; // System program
    let rent_sysvar = next_account_info(accounts_iter)?; // Rent sysvar

    // Derive PDAs
    let (user_pda, _user_bump) = Pubkey::find_program_address(&[username.as_bytes()], program_id);
    let (user_token_pda, token_bump) =
        Pubkey::find_program_address(&[username.as_bytes(), b"token"], program_id);

    if user_token_account.key != &user_token_pda {
        msg!("ERROR: Invalid PDA provided for user token account");
        return Err(ProgramError::InvalidArgument);
    }

    if mint_account.data_is_empty() {
        msg!("ERROR: Mint account not initialized!");
        return Err(ProgramError::UninitializedAccount);
    }

    msg!("Creating token account for user: {}", username);
    let rent = Rent::get()?;
    let token_acc_size = spl_token::state::Account::LEN;
    let lamports = rent.minimum_balance(token_acc_size);

    // PDA signs via seeds
    invoke_signed(
        &system_instruction::create_account(
            payer.key,
            user_token_account.key,
            lamports,
            token_acc_size as u64,
            token_program.key,
        ),
        &[
            payer.clone(),
            user_token_account.clone(),
            system_program.clone(),
        ],
        &[&[username.as_bytes(), b"token", &[token_bump]]],
    )?;

    msg!("Initializing token account...");
    invoke(
        &spl_token::instruction::initialize_account(
            token_program.key,
            user_token_account.key,
            mint_account.key,
            &user_pda, // owner
        )?,
        &[
            user_token_account.clone(),
            mint_account.clone(),
            payer.clone(),
            token_program.clone(),
            rent_sysvar.clone(),
        ],
    )?;

    msg!(" User token account created successfully!");
    Ok(())
}

pub fn mint_to_user(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    username: String,
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let mint_account = next_account_info(accounts_iter)?;
    let user_token_account = next_account_info(accounts_iter)?;
    let mint_authority = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    invoke(
        &token_instruction::mint_to(
            token_program.key,
            mint_account.key,
            user_token_account.key,
            mint_authority.key,
            &[],
            amount,
        )?,
        &[
            mint_account.clone(),
            user_token_account.clone(),
            mint_authority.clone(),
            token_program.clone(),
        ],
    )?;

    msg!("Minted {} tokens to user {}", amount, username);
    Ok(())
}

pub fn transfer_to_user(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    from_username: String,
    to_username: String,
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let from_token_account = next_account_info(accounts_iter)?;
    let to_token_account = next_account_info(accounts_iter)?;
    let authority = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    invoke(
        &token_instruction::transfer(
            token_program.key,
            from_token_account.key,
            to_token_account.key,
            authority.key,
            &[],
            amount,
        )?,
        &[
            from_token_account.clone(),
            to_token_account.clone(),
            authority.clone(),
            token_program.clone(),
        ],
    )?;
    msg!(
        "Transferred {} tokens from {} to {}",
        amount,
        from_username,
        to_username
    );
    Ok(())
}
