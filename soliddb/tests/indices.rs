use serde::{Deserialize, Serialize};
use soliddb::*;
use temp_dir::TempDir;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Table)]
#[solid(table = 1)]
struct User {
    #[solid(unique)]
    name: String,
    #[solid(indexed)]
    group: String,
}

#[test]
fn unique_is_unique() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let db = soliddb::open(dir.path())?;

    let user = User {
        name: "pako".to_string(),
        group: "users".to_string(),
    };
    user.create(&db)?;
    user.create(&db).unwrap_err();

    Ok(())
}

#[test]
fn get_by_indices() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let db = soliddb::open(dir.path())?;

    for i in 0..10 {
        let user = User {
            name: format!("pako{i}"),
            group: "users".to_string(),
        };

        user.create(&db)?;
    }

    for i in 0..5 {
        let user = User {
            name: format!("simon{i}"),
            group: "admins".to_string(),
        };
        user.create(&db)?;
    }

    let users1 = User::get_by_group(&db, &"users".to_string())?;
    assert_eq!(users1.len(), 10);

    let users2 = User::get_by_group(&db, &"admins".to_string())?;
    assert_eq!(users2.len(), 5);

    Ok(())
}
