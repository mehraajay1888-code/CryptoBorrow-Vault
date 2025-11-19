#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, log, symbol_short, Address, Env, Symbol};

// Struct to track loan details
#[contracttype]
#[derive(Clone)]
pub struct Loan {
    pub borrower: Address,
    pub collateral_amount: i128,
    pub borrowed_amount: i128,
    pub is_active: bool,
    pub timestamp: u64,
}

// Mapping borrower address to their loan
#[contracttype]
pub enum LoanBook {
    Loan(Address),
}

// Symbol for tracking total loans count
const TOTAL_LOANS: Symbol = symbol_short!("T_LOANS");

// Symbol for tracking total collateral in vault
const TOTAL_COLLATERAL: Symbol = symbol_short!("T_COLL");

#[contract]
pub struct CryptoBorrowVault;

#[contractimpl]
impl CryptoBorrowVault {
    /// Deposit collateral and borrow tokens (70% of collateral value)
    /// collateral_amount: Amount of tokens to deposit as collateral
    pub fn borrow(env: Env, borrower: Address, collateral_amount: i128) -> i128 {
        borrower.require_auth();

        // Check if borrower already has an active loan
        let existing_loan = Self::view_loan(env.clone(), borrower.clone());

        if existing_loan.is_active {
            log!(&env, "Borrower already has an active loan");
            panic!("Active loan exists. Repay first!");
        }

        // Calculate borrowable amount (70% of collateral)
        let borrowed_amount = (collateral_amount * 70) / 100;

        // Create new loan
        let loan = Loan {
            borrower: borrower.clone(),
            collateral_amount,
            borrowed_amount,
            is_active: true,
            timestamp: env.ledger().timestamp(),
        };

        // Update total loans counter
        let mut total_loans: i128 = env.storage().instance().get(&TOTAL_LOANS).unwrap_or(0);
        total_loans += 1;
        env.storage().instance().set(&TOTAL_LOANS, &total_loans);

        // Update total collateral in vault
        let mut total_collateral: i128 =
            env.storage().instance().get(&TOTAL_COLLATERAL).unwrap_or(0);
        total_collateral += collateral_amount;
        env.storage()
            .instance()
            .set(&TOTAL_COLLATERAL, &total_collateral);

        // Store loan details
        env.storage()
            .instance()
            .set(&LoanBook::Loan(borrower.clone()), &loan);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(
            &env,
            "Loan created: Collateral {}, Borrowed {}",
            collateral_amount,
            borrowed_amount
        );

        borrowed_amount
    }

    /// Repay the loan and retrieve collateral
    pub fn repay(env: Env, borrower: Address) {
        borrower.require_auth();

        // Get existing loan
        let mut loan = Self::view_loan(env.clone(), borrower.clone());

        if !loan.is_active {
            log!(&env, "No active loan found");
            panic!("No active loan to repay!");
        }

        // Mark loan as inactive (repaid)
        loan.is_active = false;

        // Update total collateral in vault
        let mut total_collateral: i128 =
            env.storage().instance().get(&TOTAL_COLLATERAL).unwrap_or(0);
        total_collateral -= loan.collateral_amount;
        env.storage()
            .instance()
            .set(&TOTAL_COLLATERAL, &total_collateral);

        // Update loan status
        env.storage()
            .instance()
            .set(&LoanBook::Loan(borrower.clone()), &loan);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(
            &env,
            "Loan repaid. Collateral {} returned",
            loan.collateral_amount
        );
    }

    /// View loan details for a specific borrower
    pub fn view_loan(env: Env, borrower: Address) -> Loan {
        let key = LoanBook::Loan(borrower.clone());

        env.storage().instance().get(&key).unwrap_or(Loan {
            borrower: borrower.clone(),
            collateral_amount: 0,
            borrowed_amount: 0,
            is_active: false,
            timestamp: 0,
        })
    }

    /// View vault statistics
    pub fn view_vault_stats(env: Env) -> (i128, i128) {
        let total_loans: i128 = env.storage().instance().get(&TOTAL_LOANS).unwrap_or(0);
        let total_collateral: i128 = env.storage().instance().get(&TOTAL_COLLATERAL).unwrap_or(0);

        (total_loans, total_collateral)
    }
}
