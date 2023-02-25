use serde::{Deserialize, Serialize};
use soliddb::*;
use temp_dir::TempDir;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Table)]
#[solid(table = 1)]
enum User {
    Created { name: String, pass: String },
    Disabled { name: String, pass: String },
}

#[test]
fn simple() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let db = soliddb::open(dir.path())?;

    let user = User::Created {
        name: "pako".to_string(),
        pass: "123".to_string(),
    };

    let id = user.create(&db)?;
    let with_id = User::get(&db, id)?;
    assert_eq!(with_id.id, id);
    assert_eq!(with_id.value, user);

    let user = User::Disabled {
        name: "pako".to_string(),
        pass: "123".to_string(),
    };

    user.update(&db, id)?;
    let with_id = User::get(&db, id)?;
    assert_eq!(with_id.id, id);
    assert_eq!(with_id.value, user);

    User::delete(&db, id)?;
    User::get(&db, id).unwrap_err();

    Ok(())
}
