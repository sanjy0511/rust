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

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserAccount {
    pub username: String,
    pub email: String,
    pub password: String,
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
}

#[cfg(not(feature = "client"))]
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Entrypoint triggered");

    let instruction = UserInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        UserInstruction::Signup {
            username,
            email,
            password,
        } => signup(program_id, accounts, username, email, password),
        UserInstruction::Signin { username, password } => signin(accounts, username, password),
    }
}

// ---------------- Signup ----------------
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
    let system_program = next_account_info(accounts_iter)?;

    let user_data = UserAccount {
        username: username.clone(),
        email: email.clone(),
        password,
    };
    let serialized_size = user_data.try_to_vec()?.len();
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(serialized_size);

    let (pda, bump) = Pubkey::find_program_address(&[username.as_bytes()], program_id);

    if user_account.key != &pda {
        msg!("Invalid PDA provided");
        return Err(ProgramError::InvalidArgument);
    }

    if user_account.data_is_empty() {
        let create_ix = system_instruction::create_account(
            payer.key,
            &pda,
            lamports,
            serialized_size as u64,
            program_id,
        );

        invoke_signed(
            &create_ix,
            &[payer.clone(), user_account.clone(), system_program.clone()],
            &[&[username.as_bytes(), &[bump]]],
        )?;
    }

    user_account.data.borrow_mut().fill(0);
    user_data.serialize(&mut &mut user_account.data.borrow_mut()[..])?;

    msg!("Signup success! Username: {}, Email: {}", username, email);
    Ok(())
}

// ---------------- Signin ----------------
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
