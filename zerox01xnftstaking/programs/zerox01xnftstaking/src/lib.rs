pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("8VattfYn7VfwWtiWuoVBmrFbGybcpG3G61VH7XG4Uo8d");

#[program]
pub mod zerox01xnftstaking {
    use super::*;
    
    pub fn initialize(ctx : Context<Initialize> , reward_per_second : u64 ) -> Result<() >{
        initialize::handler(ctx, reward_per_second)
    }
  
}
