use rusqlite::Connection;

pub fn init(conn: &mut Connection) {
    conn.pragma_update(None, "journal_mode", "WAL").unwrap();
    conn.pragma_update(None, "synchronous", "NORMAL").unwrap();
    let trans = conn.transaction().unwrap();

    // Init tables
    for i in [
        include_str!("sql/create_facts.sql"),
        include_str!("sql/create_users.sql"),
    ] {
        trans.execute(i, []).unwrap();
    }
    trans.commit().unwrap();
}
