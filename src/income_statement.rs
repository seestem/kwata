//! # Get Income Statement kontroller

use crate::INCOME_STATEMENTS_DB;
use finspider::income_statements::{database::IncomeStatementsDB, IncomeStatement};
use finspider::Spider;
use kong::{server, ErrorResponse, Kong, Kontrol, Method};
use postgres::Client as PgClient;
use std::sync::{Arc, Mutex};

/// ✨ Get income statements kontroller
pub struct GetIncomeStatementsKontroller {
    /// Address to kontroller
    pub address: String,
    /// HTTP method supported by the kontroller
    pub method: Method,
    /// PostgreSQL database handle
    pub database: Arc<Mutex<PgClient>>,
}

impl GetIncomeStatementsKontroller {
    /// Process the result from the client
    fn process_client_result(
        income_statements: Vec<IncomeStatement>,
        db: &mut PgClient,
    ) -> server::Response {
        if income_statements.is_empty() {
            server::Response::json(&income_statements)
        } else {
            // Store Income Statements in db
            for income_statement in income_statements.iter() {
                let input = income_statement;

                let db_res = IncomeStatementsDB::save(db, INCOME_STATEMENTS_DB, input.clone());

                if db_res.is_err() {
                    return ErrorResponse::internal();
                }
            }
            server::Response::json(&income_statements)
        }
    }
}

impl Kontrol for GetIncomeStatementsKontroller {
    /// Endpoint's address
    fn address(&self) -> String {
        self.address.clone()
    }

    /// Method supported by endpoint
    fn method(&self) -> Method {
        self.method
    }

    /// Retrieve income statement
    fn kontrol(&self, kong: &Kong) -> server::Response {
        if let Some(url_params) = &kong.url_parameters {
            if let Some(symbol) = url_params.find("symbol") {
                // Get income statement from database
                let mut db = self.database.lock().unwrap();
                let db_result =
                    IncomeStatementsDB::read_all_by_symbol(&mut db, INCOME_STATEMENTS_DB, symbol);

                match db_result {
                    Ok(income_statements) => {
                        if income_statements.is_empty() {
                            // No income statements found in the db
                            // try fetching upstream

                            let html = IncomeStatement::fetch(symbol).unwrap();
                            let res: Vec<IncomeStatement> = IncomeStatement::parse(&html, symbol);

                            GetIncomeStatementsKontroller::process_client_result(res, &mut db)
                        } else {
                            // Income statements found in db
                            server::Response::json(&income_statements)
                        }
                    }
                    Err(_) => ErrorResponse::internal(),
                }
            } else {
                ErrorResponse::bad_request()
            }
        } else {
            ErrorResponse::bad_request()
        }
    }
}
