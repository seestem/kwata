//! # Get Balance Sheets kontroller

use crate::BALANCE_SHEETS_DB;
use finspider::balance_sheets::{database::BalanceSheetsDB, BalanceSheet};
use finspider::Spider;
use kong::{server, ErrorResponse, Kong, Kontrol, Method};
use postgres::Client as PgClient;
use std::sync::{Arc, Mutex};

/// Get balance sheets kontroller
pub struct GetBalanceSheetsKontroller {
    /// Address to kontroller
    pub address: String,
    /// HTTP method supported by the kontroller
    pub method: Method,
    /// PostgreSQL database handle
    pub database: Arc<Mutex<PgClient>>,
}

impl GetBalanceSheetsKontroller {
    /// Process the result from the client
    fn process_client_result(
        balance_sheets: Vec<BalanceSheet>,
        db: &mut PgClient,
    ) -> server::Response {
        if balance_sheets.is_empty() {
            server::Response::json(&balance_sheets)
        } else {
            // Store Balance Sheets in db
            for income_statement in balance_sheets.iter() {
                let input = income_statement;

                let db_res = BalanceSheetsDB::save(db, BALANCE_SHEETS_DB, input.clone());

                if db_res.is_err() {
                    return ErrorResponse::internal();
                }
            }
            server::Response::json(&balance_sheets)
        }
    }
}

impl Kontrol for GetBalanceSheetsKontroller {
    /// Endpoint's address
    fn address(&self) -> String {
        self.address.clone()
    }

    /// Method supported by endpoint
    fn method(&self) -> Method {
        self.method
    }

    /// Retrieve balance sheets
    fn kontrol(&self, kong: &Kong) -> server::Response {
        if let Some(url_params) = &kong.url_parameters {
            if let Some(symbol) = url_params.find("symbol") {
                // Get balance sheets from database
                let mut db = self.database.lock().unwrap();
                let db_result =
                    BalanceSheetsDB::read_all_by_symbol(&mut db, BALANCE_SHEETS_DB, symbol);

                match db_result {
                    Ok(balance_sheets) => {
                        if balance_sheets.is_empty() {
                            // No balance sheets found in the db
                            // try fetching upstream

                            let html = BalanceSheet::fetch(symbol).unwrap();
                            let res: Vec<BalanceSheet> = BalanceSheet::parse(&html, &symbol);

                            GetBalanceSheetsKontroller::process_client_result(res, &mut db)
                        } else {
                            // Balance sheets found in db
                            server::Response::json(&balance_sheets)
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
