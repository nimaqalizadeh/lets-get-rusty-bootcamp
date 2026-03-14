use std::rc::Rc;
use std::cell::RefCell;

struct Database {
    max_connections: u32
}

struct AuthService {
    db: Rc<RefCell<Database>>
}

struct ContentService {
    db: Rc<RefCell<Database>>
}

fn main() {
    let db = Rc::new(RefCell::new(Database {
        max_connections: 100
    }));
    let auth_service = AuthService { db: Rc::clone(&db) };
    let content_service = ContentService { db: Rc::clone(&db) };
    
    // BUG: This compiles fine but panics at runtime.
    // r1 takes a mutable borrow of db. Before r1 is released, r2 tries to
    // take another mutable borrow — RefCell detects this violation at runtime
    // and panics with: "already mutably borrowed".
    // RefCell enforces Rust's borrow rules (at most one mutable OR any number
    // of immutable borrows at a time), just at runtime instead of compile time.
    let mut r1 = db.borrow_mut();
    let r2 = db.borrow_mut(); // <-- runtime panic here
    r1.max_connections = 200;
}
