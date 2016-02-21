# iron-postgres-middleware

An attempt to create postgres middleware for the [Iron](https://github.com/iron/iron/) web framework

## Usage

### Cargo.toml

```toml
[dependencies.iron-postgres-middleware]
git = "https://github.com/martinsp/iron-postgres-middleware.git"
```

### Import

```rust
extern crate iron_postgres_middleware as pg_middleware;
use pg_middleware::{PostgresMiddleware, PostgresReqExt};
```

### Middleware

```rust
fn main() {
  let mut router = Router::new();
  router.get("/", handler);

  let mut c = Chain::new(router);

  let pg_middleware = PostgresMiddleware::new("postgres://user@localhost/db_name").unwrap();
  c.link_before(pg_middleware);

  Iron::new(c).http("localhost:3000").unwrap();
}

fn handler(req: &mut Request) -> IronResult<Response> {
  let con = req.db_conn();
  con.execute("INSERT INTO foo (bar) VALUES ($1)", &[&1i32]).unwrap();
  Ok(Response::new().set(status::Ok).set("Success"))
}
```
