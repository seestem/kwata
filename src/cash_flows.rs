//! # Get Cash Flows kontroller

use crate::CASH_FLOWS_DB;
use finspider::cash_flows::{database::CashFlowDB, CashFlow};
use finspider::Spider;
use kong::{server, ErrorResponse, Kong, Kontrol, Method};
use postgres::Client as PgClient;
use std::sync::{Arc, Mutex};

/// Get cash flows kontroller
pub struct GetCashFlowsKontroller {
    /// Address to kontroller
    pub address: String,
    /// HTTP method supported by the kontroller
    pub method: Method,
    /// PostgreSQL database handle
    pub database: Arc<Mutex<PgClient>>,
}

impl GetCashFlowsKontroller {
    /// Process the result from the client
    fn process_client_result(cash_flows: Vec<CashFlow>, db: &mut PgClient) -> server::Response {
        if cash_flows.is_empty() {
            server::Response::json(&cash_flows)
        } else {
            // Store Cash flowss in db
            for cash_flow in cash_flows.iter() {
                let input = cash_flow;

                let db_res = CashFlowDB::save(db, CASH_FLOWS_DB, input.clone());

                if db_res.is_err() {
                    return ErrorResponse::internal();
                }
            }
            server::Response::json(&cash_flows)
        }
    }
}

impl Kontrol for GetCashFlowsKontroller {
    /// Endpoint's address
    fn address(&self) -> String {
        self.address.clone()
    }

    /// Method supported by endpoint
    fn method(&self) -> Method {
        self.method
    }

    /// Retrieve cash statement
    fn kontrol(&self, kong: &Kong) -> server::Response {
        if let Some(url_params) = &kong.url_parameters {
            if let Some(symbol) = url_params.find("symbol") {
                // Get cash statement from database
                let mut db = self.database.lock().unwrap();
                let db_result = CashFlowDB::read_all_by_symbol(&mut db, CASH_FLOWS_DB, symbol);

                match db_result {
                    Ok(cash_flows) => {
                        if cash_flows.is_empty() {
                            // No cash flows found in the db
                            // try fetching upstream

                            let html = CashFlow::fetch(symbol).unwrap();
                            let res: Vec<CashFlow> = CashFlow::parse(&html, symbol);

                            GetCashFlowsKontroller::process_client_result(res, &mut db)
                        } else {
                            // Cash flows found in db
                            server::Response::json(&cash_flows)
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
