use serde::{Deserialize, Serialize};
use soliddb::*;
use temp_dir::TempDir;
use ulid::Ulid;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Table)]
#[solid(table = 1)]
struct User {
    name: String,
    pass: String,
}

#[test]
fn simple() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let db = soliddb::open(dir.path())?;

    let user = User {
        name: "pako".to_string(),
        pass: "123".to_string(),
    };

    let id = user.create(&db)?;
    User::delete(&db, id)?;
    let id = user.create(&db)?;

    let with_id = User::get(&db, id)?;
    assert_eq!(with_id.value, user);
    assert_eq!(with_id.id, id);

    Ok(())
}

#[test]
fn get_and_delete() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let db = soliddb::open(dir.path())?;

    let user = User {
        name: "pako".to_string(),
        pass: "123".to_string(),
    };

    let id = user.create(&db)?;
    let with_id = User::get(&db, id)?;
    assert_eq!(with_id.id, id);
    assert_eq!(with_id.value, user);

    User::delete(&db, id)?;
    let err = User::get(&db, id).unwrap_err();
    assert!(matches!(err, Error::NotFound));

    Ok(())
}

#[test]
fn update_existing_item() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let db = soliddb::open(dir.path())?;

    let user = User {
        name: "pako".to_string(),
        pass: "123".to_string(),
    };
    let id = user.create(&db)?;

    let user = User {
        name: "pako1".to_string(),
        pass: "1234".to_string(),
    };
    user.update(&db, id)?;

    let with_id = User::get(&db, id)?;
    assert_eq!(with_id.value, user);

    Ok(())
}

#[test]
fn get_many() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let db = soliddb::open(dir.path())?;

    let user1 = User {
        name: "pako".to_string(),
        pass: "123".to_string(),
    };

    let user2 = User {
        name: "simon".to_string(),
        pass: "123".to_string(),
    };

    let user3 = User {
        name: "niko".to_string(),
        pass: "123".to_string(),
    };

    let id1 = user1.create(&db)?;
    let id2 = user2.create(&db)?;
    let id3 = user3.create(&db)?;

    let users = User::get_many(&db, &[id1, id2, id3])?;
    assert_eq!(users[0].value, user1);
    assert_eq!(users[1].value, user2);
    assert_eq!(users[2].value, user3);

    Ok(())
}

#[test]
fn get_many_non_existing() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let db = soliddb::open(dir.path())?;

    let user1 = User {
        name: "pako".to_string(),
        pass: "123".to_string(),
    };

    let user2 = User {
        name: "simon".to_string(),
        pass: "123".to_string(),
    };

    let user3 = User {
        name: "niko".to_string(),
        pass: "123".to_string(),
    };

    let id1 = user1.create(&db)?;
    let id2 = user2.create(&db)?;
    let id3 = user3.create(&db)?;

    let non_existent = Ulid::new();

    let err = User::get_many(&db, &[id1, non_existent, id2, id3]).unwrap_err();
    assert!(matches!(err, Error::NotFound));

    Ok(())
}

#[test]
fn iter_many() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let db = soliddb::open(dir.path())?;

    let user1 = User {
        name: "pako".to_string(),
        pass: "123".to_string(),
    };

    let user2 = User {
        name: "simon".to_string(),
        pass: "123".to_string(),
    };

    let user3 = User {
        name: "niko".to_string(),
        pass: "123".to_string(),
    };

    let id1 = user1.create(&db)?;
    std::thread::sleep(std::time::Duration::from_millis(10));
    let id2 = user2.create(&db)?;
    std::thread::sleep(std::time::Duration::from_millis(10));
    let id3 = user3.create(&db)?;

    let mut users = User::iter(&db);

    let user = users.next().unwrap()?;
    assert_eq!(user.id, id1);
    assert_eq!(user.value, user1);

    let user = users.next().unwrap()?;
    assert_eq!(user.id, id2);
    assert_eq!(user.value, user2);

    let user = users.next().unwrap()?;
    assert_eq!(user.id, id3);
    assert_eq!(user.value, user3);

    assert!(users.next().is_none());

    Ok(())
}
