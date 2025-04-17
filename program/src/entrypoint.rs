use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::PrintProgramError,
    pubkey::Pubkey,
};

use crate::{error::MTreeError, processor::Processor};

solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = Processor::process(program_id, accounts, instruction_data) {
        error.print::<MTreeError>();
        return Err(error);
    }
    Ok(())
}
