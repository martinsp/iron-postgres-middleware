extern crate iron;

extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;

use iron::prelude::*;
use iron::{typemap, BeforeMiddleware};

use std::sync::Arc;
use std::default::Default;
use postgres::{SslMode};
use r2d2_postgres::PostgresPoolManager;

pub struct PostgresMiddleware {
  pub pool: Arc<r2d2::Pool<postgres::Connection, r2d2_postgres::Error,
    r2d2_postgres::PostgresPoolManager, r2d2::LoggingErrorHandler>>,
}

struct Value(Arc<r2d2::Pool<postgres::Connection, r2d2_postgres::Error,
          r2d2_postgres::PostgresPoolManager, r2d2::LoggingErrorHandler>>);

impl typemap::Key for PostgresMiddleware { type Value = Value; }

impl PostgresMiddleware {
  pub fn new(pg_connection_str: &str) -> PostgresMiddleware {
    let config = Default::default();
    let manager = PostgresPoolManager::new(pg_connection_str, SslMode::None);
    let error_handler = r2d2::LoggingErrorHandler;
    let pool = Arc::new(r2d2::Pool::new(config, manager, error_handler).unwrap());
    PostgresMiddleware {
      pool: pool,
    }
  }
}

impl BeforeMiddleware for PostgresMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<PostgresMiddleware>(Value(self.pool.clone()));
        Ok(())
    }
}

pub trait PostgresReqExt {
  fn db_conn(&self) -> r2d2::PooledConnection<postgres::Connection, r2d2_postgres::Error,
    r2d2_postgres::PostgresPoolManager, r2d2::LoggingErrorHandler>;
}

impl PostgresReqExt for Request {
  fn db_conn(&self) -> r2d2::PooledConnection<postgres::Connection, r2d2_postgres::Error,
    r2d2_postgres::PostgresPoolManager, r2d2::LoggingErrorHandler> {
    let poll_value = self.extensions.get::<PostgresMiddleware>().unwrap();
    let &Value(ref poll) = poll_value;

    return poll.get().unwrap();
  }
}
