#![feature(core)]

extern crate iron;
extern crate postgres;
extern crate "iron-postgres-middleware" as pg_middleware;

use iron::prelude::*;
use iron::status;
use pg_middleware::{PostgresMiddleware, PostgresReqExt};

fn main() {
    let mut chain = Chain::new(name_list);

    let pg_middleware = PostgresMiddleware::new("postgres://postgres@localhost/example");

    {
        let conn = pg_middleware.pool.get().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS names (
                id SERIAL PRIMARY KEY,
                name VARCHAR(255) NOT NULL
            )",
        &[]).unwrap();
        conn.execute("INSERT INTO names(name) VALUES ($1)", &[&"Joe Smith".to_string()]).unwrap();
    }

    chain.link_before(pg_middleware);

    Iron::new(chain).listen("localhost:3000").unwrap();
}

fn name_list(req: &mut Request) -> IronResult<Response> {
    let conn = req.db_conn();
    let stmt = conn.prepare("SELECT id, name FROM names").unwrap();
    let rows = stmt.query(&[]).unwrap();

    let mut resp_str = "Names:\n".to_string();

    for row in rows {
        let id: i32 = row.get(0);
        let name: String = row.get(1);
        let name_format = format!("{}: {}\n", id, name);
        resp_str.push_str(name_format.as_slice());
    }

    Ok(Response::with((status::Ok, resp_str)))
}
