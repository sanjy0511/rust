use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(not(feature = "client"))]
use solana_program::entrypoint;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
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
    }
}

pub fn signup(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    username: String,
    email: String,
    password: String,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer = next_account_info(accounts_iter)?; // creating the user account.
    let user_account = next_account_info(accounts_iter)?; // Retrieves the account where user-specific data will be stored.
    let registry_account = next_account_info(accounts_iter)?; //Refers to a central registry account that keeps track of all users.
    let system_program = next_account_info(accounts_iter)?; //Needed when you want to create new accounts or transfer SOL.

    let user_data = UserAccount {
        username: username.clone(),
        email: email.clone(),
        password,
    };

    let (user_pda, user_bump) = Pubkey::find_program_address(&[username.as_bytes()], program_id);
    //PDAs are special addresses controlled by your program, not by a private key.
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
        //User PDA => user-ku unique account, store personal data.
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
        //Registry PDA => Single account, track ellam user PDAs globally.

        let empty = UserRegistry::default();
        empty.serialize(&mut &mut registry_account.data.borrow_mut()[..])?;
        msg!("Empty registry initialized!");
    }

    // Copy data out first to avoid double borrow
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
