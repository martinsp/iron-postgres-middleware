extern crate iron;

extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;

use iron::prelude::*;
use iron::{typemap, BeforeMiddleware};

use std::sync::Arc;
use std::default::Default;
use postgres::{SslMode};
use r2d2_postgres::PostgresConnectionManager;

/// Iron middleware that allows for postgres connections within requests.
pub struct PostgresMiddleware {
  /// A pool of postgres connections that are shared between requests.
  pub pool: Arc<r2d2::Pool<r2d2_postgres::PostgresConnectionManager>>,
}

struct Value(Arc<r2d2::Pool<r2d2_postgres::PostgresConnectionManager>>);

impl typemap::Key for PostgresMiddleware { type Value = Value; }

impl PostgresMiddleware {

  /// Creates a new pooled connection to the given postgresql server. The URL is in the format:
  ///
  /// ```{none}
  /// postgresql://user[:password]@host[:port][/database][?param1=val1[[&param2=val2]...]]
  /// ```
  ///
  /// **Panics** if there are any errors connecting to the postgresql database.
  pub fn new(pg_connection_str: &str) -> PostgresMiddleware {
    let config = Default::default();
    let manager = PostgresConnectionManager::new(pg_connection_str, SslMode::None);
    let error_handler = r2d2::LoggingErrorHandler;
    let pool = Arc::new(r2d2::Pool::new(config, manager, Box::new(error_handler)).unwrap());
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

/// Adds a method to requests to get a database connection.
///
/// ## Example
///
/// ```ignore
/// fn handler(req: &mut Request) -> IronResult<Response> {
///   let conn = req.db_conn();
///   con.execute("INSERT INTO foo (bar) VALUES ($1)", &[&1i32]).unwrap();
///
///   Ok(Response::with((status::Ok, resp_str)))
/// }
/// ```
pub trait PostgresReqExt {
  /// Returns a pooled connection to the postgresql database. The connection is returned to
  /// the pool when the pooled connection is dropped.
  fn db_conn(&self) -> r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager>;
}

impl<'a> PostgresReqExt for Request<'a> {
  fn db_conn(&self) -> r2d2::PooledConnection<r2d2_postgres::PostgresConnectionManager> {
    let poll_value = self.extensions.get::<PostgresMiddleware>().unwrap();
    let &Value(ref poll) = poll_value;

    return poll.get().unwrap();
  }
}
