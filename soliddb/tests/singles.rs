use serde::{Deserialize, Serialize};
use soliddb::Single;
use temp_dir::TempDir;

#[derive(Debug, PartialEq, Serialize, Deserialize, Single)]
#[solid(single = 1)]
struct Config {
    some_field: String,
    some_other_field: u64,
    another_field: Vec<f32>,
}

#[test]
fn put_and_delete_single() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let db = soliddb::open(dir.path())?;

    let config = Config {
        some_field: "pako".to_string(),
        some_other_field: 13,
        another_field: vec![],
    };

    config.put(&db)?;
    let got = Config::get(&db)?;
    assert_eq!(got, config);

    Config::delete(&db)?;
    Config::get(&db).unwrap_err();

    Ok(())
}

#[test]
fn overwrite() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let db = soliddb::open(dir.path())?;

    let config = Config {
        some_field: "pako".to_string(),
        some_other_field: 13,
        another_field: vec![],
    };
    config.put(&db)?;

    let new_config = Config {
        some_field: "pako1".to_string(),
        some_other_field: 42,
        another_field: vec![123.45],
    };
    new_config.put(&db)?;

    let got = Config::get(&db)?;
    assert_eq!(got, new_config);

    Ok(())
}
