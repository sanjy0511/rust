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

/// ---------------- USER STRUCT ----------------
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserAccount {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// ---------------- REGISTRY STRUCT ----------------
#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct UserRegistry {
    pub users: Vec<Pubkey>,
}

/// ---------------- INSTRUCTION ENUM ----------------
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

/// ---------------- ENTRYPOINT ----------------
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

/// ---------------- SIGNUP FUNCTION ----------------
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

    // Prepare user data
    let user_data = UserAccount {
        username: username.clone(),
        email: email.clone(),
        password,
    };

    let user_serialized_size = user_data.try_to_vec()?.len();
    let rent = Rent::get()?;
    let user_lamports = rent.minimum_balance(user_serialized_size);

    let (user_pda, user_bump) = Pubkey::find_program_address(&[username.as_bytes()], program_id);
    if user_account.key != &user_pda {
        msg!("Invalid PDA provided for user");
        return Err(ProgramError::InvalidArgument);
    }

    // Create user account if not exists
    if user_account.data_is_empty() {
        msg!("Creating user PDA account...");
        let create_user_ix = system_instruction::create_account(
            payer.key,
            &user_pda,
            user_lamports,
            user_serialized_size as u64,
            program_id,
        );
        invoke_signed(
            &create_user_ix,
            &[payer.clone(), user_account.clone(), system_program.clone()],
            &[&[username.as_bytes(), &[user_bump]]],
        )?;
    }

    // Write user data
    user_account.data.borrow_mut().fill(0);
    user_data.serialize(&mut &mut user_account.data.borrow_mut()[..])?;
    msg!(" Signup success! Username: {}, Email: {}", username, email);

    // ---------------- REGISTRY LOGIC ----------------
    let (registry_pda, registry_bump) = Pubkey::find_program_address(&[b"registry"], program_id);
    if registry_account.key != &registry_pda {
        msg!("Invalid registry PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let rent = Rent::get()?;
    let registry_serialized_size = 4096;
    let registry_lamports = rent.minimum_balance(registry_serialized_size);

    // Create registry if not exists
    if registry_account.data_is_empty() {
        msg!("Creating registry PDA account...");
        let create_registry_ix = system_instruction::create_account(
            payer.key,
            &registry_pda,
            registry_lamports,
            registry_serialized_size as u64,
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

        let empty_registry = UserRegistry::default();
        empty_registry.serialize(&mut &mut registry_account.data.borrow_mut()[..])?;
        msg!("Initialized empty registry");
    }

    // Read registry safely
    let mut registry: UserRegistry =
        UserRegistry::try_from_slice(&registry_account.data.borrow()).unwrap_or_default();

    // Avoid duplicates
    if !registry.users.contains(&user_pda) {
        registry.users.push(user_pda);
    }

    // Clean buffer before writing
    registry_account.data.borrow_mut().fill(0);
    registry.serialize(&mut &mut registry_account.data.borrow_mut()[..])?;
    msg!("Registry updated successfully");

    Ok(())
}

/// ---------------- SIGNIN FUNCTION ----------------
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

/// ---------------- LIST USERS FUNCTION ----------------
pub fn list_users(accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let registry_account = next_account_info(accounts_iter)?;

    let registry: UserRegistry =
        UserRegistry::try_from_slice(&registry_account.data.borrow()).unwrap_or_default();

    if registry.users.is_empty() {
        msg!("No users registered yet.");
        return Ok(());
    }

    msg!("-------- Registered Users --------");
    for user_pda in registry.users.iter() {
        msg!("User PDA: {}", user_pda);
    }
    Ok(())
}
