use std::time::Duration;

use serde::{Deserialize, Serialize};
use soliddb::*;
use temp_dir::TempDir;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Table)]
#[solid(table = 1)]
struct UserV1 {
    #[solid(unique)]
    name: String,
    pass: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Table)]
#[solid(table = 1)]
struct UserV2 {
    #[solid(unique)]
    name: String,
    pass: String,
    extra: Option<String>,
    #[serde(default)]
    more: Vec<String>,
}

#[test]
fn add_optional_field_and_more() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let db = soliddb::open(dir.path())?;

    let user = UserV1 {
        name: "pako".to_string(),
        pass: "123".to_string(),
    };
    let id = user.create(&db)?;

    let got = UserV2::get(&db, id)?;
    assert_eq!(got.value.name, user.name);
    assert_eq!(got.value.pass, user.pass);
    assert!(got.value.extra.is_none());
    assert!(got.value.more.is_empty());
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Table)]
#[solid(table = 2)]
struct Type1 {
    name: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Table)]
#[solid(table = 2)]
struct Type2 {
    description: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Table)]
#[solid(table = 2)]
#[serde(untagged)]
enum Migratable {
    V1(Type1),
    V2(Type2),
}

#[test]
fn schema_change_with_version_enum() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let db = soliddb::open(dir.path())?;

    let t1 = Type1 {
        name: "something".to_string(),
    };

    let t2 = Type2 {
        description: "name".to_string(),
    };

    let id1 = t1.create(&db)?;
    std::thread::sleep(Duration::from_millis(10));
    let id2 = t2.create(&db)?;

    let all = Migratable::all(&db)?;
    assert_eq!(all[0].id, id1);
    assert_eq!(all[0].value, Migratable::V1(t1));
    assert_eq!(all[1].id, id2);
    assert_eq!(all[1].value, Migratable::V2(t2));

    Ok(())
}
