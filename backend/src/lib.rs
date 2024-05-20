mod app;
mod tui;
use anyhow::{anyhow, Context};
use pyo3::{pyclass, pymethods, pymodule, types::PyModule, Bound, PyObject, PyResult};
use serde_json::{Map, Value};
use sqlx::{query_scalar, SqlitePool};
use std::{
    future::Future,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
};
use tokio::runtime::Runtime;
use tracing::debug;

#[pymodule]
fn delta_db(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ConfigStore>()?;
    Ok(())
}

/// The config store class provides configuration storage and search.
/// The configurations are stored in a sqlite database.
///
/// The config store stores config files, (yaml, yml and Json). If the file structure is changed,
/// then the config's version is bumped.
///
/// The config store also stores all versions of the config, where just the filds of a config
/// are changed, but not the values. These can be searched through using either the cli
/// or the methods `get_deltas`
///
/// :param url (str): The sqlite url: sqlite::memory: for in memory storage,
/// sqlite:///ablsoute/path/to/db/file
/// sqlite://relative/path/to/db/file
#[pyclass]
pub struct ConfigStore {
    s: Store,
}
#[pymethods]
impl ConfigStore {
    /// Construct a new config store, or connect to
    /// an existing one at a given url.
    #[new]
    fn new(url: &str) -> PyResult<ConfigStore> {
        let s = Store::new(url)?;
        Ok(ConfigStore { s })
    }

    /// Add a new config by path.
    fn add_config(&self, path: PathBuf) -> PyResult<()> {
        if !(path.is_file() || path.is_symlink()) {
            return Err(anyhow!("{} doesn't exist", path.display()).into());
        }
        let json = read_file(&path)?;
        self.s
            .add_config(path.clone().file_name().unwrap().to_str().unwrap(), json)?;
        Ok(())
    }
    /// Get all base configurations and their available versions.
    ///
    /// :return configs list[tuple[str, int]]: List of configs
    ///  as (name, version) tuples.
    fn get_configs(&self) -> PyResult<Vec<(String, i64)>> {
        Ok(self.s.get_base_configs()?)
    }
    /// Get all the configurations for a specific base config.
    ///
    /// :param name (str): Config's name
    /// :param version (Optional[int]): Version of the config, if None get latest.
    #[pyo3(signature = (name, version = None))]
    fn get_deltas(&self, name: &str, version: Option<i64>) -> PyResult<()> {
        todo!()
    }
    fn get_latest_config(&self, name: &str, version: Option<i64>) -> PyResult<Option<PyObject>> {
        todo!();
        // Ok(self.s.get_latest_config(name, version))?.map(|v| v.into());
    }
}

pub fn read_file(path: impl AsRef<Path>) -> anyhow::Result<Value> {
    let path = path.as_ref();
    let Some(f_type) = path.extension() else {
        return Err(anyhow!("Path is unparseable {}", path.display()));
    };
    let f = std::fs::File::open(path)?;
    match f_type
        .to_str()
        .ok_or(anyhow!("Path is unparseable {}", path.display()))?
    {
        "yaml" => Ok(serde_yaml::from_reader(f).context("While opening .yaml an error occured.")?),
        "json" => Ok(serde_json::from_reader(f).context("While opening .json an error occured.")?),
        "yml" => Ok(serde_yaml::from_reader(f).context("While opening .yml an error occured.")?),
        _ => Err(anyhow!(
            "File extension is not one of 'yaml', 'json', 'yml'"
        ))?,
    }
}
pub struct Store {
    pool: SqlitePool,
    rt: tokio::runtime::Runtime,
}
impl Store {
    pub fn block_on<F>(&self, f: F) -> F::Output
    where
        F: Future,
    {
        self.rt.block_on(f)
    }
    pub fn new(url: impl AsRef<str>) -> anyhow::Result<Store> {
        let rt = Runtime::new()?;
        let pool = rt.block_on(SqlitePool::connect(url.as_ref()))?;
        rt.block_on(sqlx::migrate!().run(&pool))?;
        Ok(Store { pool, rt })
    }
    fn add_base_config(
        &self,
        cfg_name: impl AsRef<str>,
        cfg: serde_json::Value,
    ) -> anyhow::Result<i64> {
        let hash = calculate_cfg_hash(&cfg)?;
        let hash_str = format!("{}", hash);
        let name = cfg_name.as_ref();
        let version = self.block_on(
            sqlx::query_scalar!(
                r#"
            INSERT INTO BaseCfgs (name, cfg, version, cfg_hash) 
            VALUES (
            $1,
            $2,
            (SELECT COUNT(*) FROM BaseCfgs WHERE BaseCfgs.name = $1), $3
            ) RETURNING version;"#,
                name,
                cfg,
                hash_str
            )
            .fetch_one(&self.pool),
        )?;
        self.block_on(
            sqlx::query!(
                r#"INSERT INTO Deltas (cfg_hash, delta) VALUES ($1, $2)"#,
                hash_str,
                Value::Null
            )
            .execute(&self.pool),
        )?;
        Ok(version)
    }

    pub fn get_base_config(
        &self,
        cfg_name: impl AsRef<str>,
        version: Option<i64>,
    ) -> anyhow::Result<Value> {
        let ref_ = cfg_name.as_ref();
        match version {
            Some(v) => self
                .block_on(
                    sqlx::query_scalar!(
                        r#"SELECT cfg as "cfg: Value" 
                    FROM BaseCfgs
                    WHERE name = $1 and version = $2;"#,
                        ref_,
                        v,
                    )
                    .fetch_one(&self.pool),
                )
                .context(format!("Query fetching {}:{} failed", ref_, v)),
            None => self
                .block_on(
                    sqlx::query_scalar!(
                        r#"SELECT cfg as "cfg: Value"
                    FROM BaseCfgs 
                    WHERE name = $1 and version = (
                    SELECT MAX(version) FROM BaseCfgs where name = $1
                    );"#,
                        ref_,
                    )
                    .fetch_one(&self.pool),
                )
                .context(format!("Query fetching {}:latest failed", ref_)),
        }
    }
    fn get_base_config_by_hash(&self, cfg_hash: u64) -> anyhow::Result<Option<Value>> {
        let hash_str = format!("{}", cfg_hash);
        let cfg = self
            .block_on(
                query_scalar!(
                    r#"SELECT cfg as "cfg: Value" FROM BaseCfgs WHERE cfg_hash = $1 LIMIT 1;"#,
                    hash_str
                )
                .fetch_optional(&self.pool),
            )
            .context("Fetching base config failed.")?;
        Ok(cfg)
    }
    pub fn add_config(
        &self,
        cfg_name: impl AsRef<str>,
        cfg: serde_json::Value,
    ) -> anyhow::Result<()> {
        let hash = calculate_cfg_hash(&cfg)?;
        let hash_str = format!("{}", hash);
        match self
            .get_base_config_by_hash(hash)
            .context("Couldn't hash config shape.")?
        {
            Some(base_cfg) => {
                debug!("Base Config found for {}", cfg_name.as_ref());
                let Some(delta) = calculate_delta(&base_cfg, &cfg) else {
                    return Ok(());
                };
                debug!("Delta found {}", &delta);
                self.block_on(
                    sqlx::query!(
                        "INSERT INTO Deltas (cfg_hash, delta) VALUES ($1, $2)",
                        hash_str,
                        delta
                    )
                    .execute(&self.pool),
                )?;
            }
            None => {
                debug!("No Base Config found for {}", cfg_name.as_ref());
                self.add_base_config(cfg_name.as_ref(), cfg)?;
            }
        };
        Ok(())
    }
    /// Fetch the config at a name and a version, if version is none, then the latest version.
    /// Return's the most recent delta commited.
    pub fn get_latest_config(
        &self,
        cfg_name: impl AsRef<str>,
        version: Option<i64>,
    ) -> anyhow::Result<Option<Value>> {
        let name = cfg_name.as_ref();
        let (hash, delta) = match version {
            Some(v) => {
                let Some(row) = self
                    .block_on(
                        sqlx::query_file!("sql/query_specific_version_cfg.sql", name, v)
                            .fetch_optional(&self.pool),
                    )
                    .context("Failed to query latest config.")?
                else {
                    return Ok(None);
                };
                (row.cfg_hash.parse::<u64>().unwrap(), row.delta)
            }
            None => {
                let Some(row) = self
                    .block_on(
                        sqlx::query_file!("sql/query_latest_version_cfg.sql", name)
                            .fetch_optional(&self.pool),
                    )
                    .context("Failed to query latest config.")?
                else {
                    return Ok(None);
                };
                (row.cfg_hash.parse::<u64>().unwrap(), row.delta)
            }
        };
        let base_cfg = self
            .get_base_config_by_hash(hash)?
            .ok_or(anyhow!("Expected to find a hash in the database."))?;
        Ok(Some(build_cfg_from_base_and_delta(base_cfg, delta)))
    }

    /// Get the base configuration types and the number of versions they have
    pub fn get_base_configs(&self) -> anyhow::Result<Vec<(String, i64)>> {
        let rows = self
            .block_on(sqlx::query!(r#"SELECT name, version FROM BaseCfgs"#).fetch_all(&self.pool));
        match rows {
            Err(e) => match e {
                sqlx::Error::RowNotFound => Ok(vec![]),
                _ => Err(e).context("An error occured during fetching configs."),
            },
            Ok(vec) => Ok(vec.into_iter().map(|r| (r.name, r.version)).collect()),
        }
    }
    pub fn get_base_config_hash(
        &self,
        cfg_name: impl AsRef<str>,
        version: Option<i64>,
    ) -> anyhow::Result<u64> {
        let cfg_name = cfg_name.as_ref();
        let string_hash = match version {
            Some(v) => self
                .block_on(
                    sqlx::query_scalar!(
                        r#"SELECT cfg_hash FROM BaseCfgs WHERE name=$1 and version = $2;"#,
                        cfg_name,
                        v
                    )
                    .fetch_one(&self.pool),
                )?,
            None => self.block_on(sqlx::query_scalar(
                r#"SELECT cfg_hash FROM BaseCfgs WHERE name=$1 and version=(SELECT MAX(version) from BaseCfgs WHERE name= $1);"#,
            ).fetch_one(&self.pool))?,
        };
        string_hash.parse().context("Failed to parse u64 hash")
    }
    pub fn get_all_deltas(
        &self,
        cfg_name: impl AsRef<str>,
        version: Option<u64>,
    ) -> anyhow::Result<Vec<(i64, Value)>> {
        let base_config_hash = self
            .get_base_config_hash(cfg_name, version.map(|i| i as i64))?
            .to_string();
        let query_result = self.block_on(
            sqlx::query!(
                r#"SELECT id, delta as "delta: Value" FROM Deltas WHERE cfg_hash = $1"#,
                base_config_hash,
            )
            .fetch_all(&self.pool),
        );
        match query_result {
            Err(e) => match e {
                sqlx::Error::RowNotFound => Ok(vec![]),
                _ => Err(e).context("Querying deltas failed."),
            },
            Ok(vec) => Ok(vec.into_iter().map(|row| (row.id, row.delta)).collect()),
        }
    }
    pub fn get_delta(&self, delta_id: i64) -> anyhow::Result<Value> {
        let row = self.block_on(
            sqlx::query!(
                r#"SELECT Deltas.delta as "delta: Value", BaseCfgs.cfg as "cfg: Value" FROM Deltas 
                    INNER JOIN BaseCfgs on Deltas.cfg_hash = BaseCfgs.cfg_hash 
                    WHERE id = $1"#,
                delta_id,
            )
            .fetch_one(&self.pool),
        )?;
        let (delta, base) = (row.delta, row.cfg);
        Ok(build_cfg_from_base_and_delta(base, delta))
    }
}
fn build_cfg_from_base_and_delta(base_cfg: Value, delta: Value) -> Value {
    match (base_cfg, delta) {
        (Value::Object(mut base_obj), Value::Object(mut delta_obj)) => {
            for (k, v) in base_obj.iter_mut() {
                if let Some(delta_value) = delta_obj.remove(k) {
                    let delta = build_cfg_from_base_and_delta(std::mem::take(v), delta_value);
                    *v = delta;
                };
            }
            Value::Object(base_obj)
        }
        (base_cfg, Value::Null) => base_cfg,
        (_, delta) => delta,
    }
}

fn calculate_cfg_hash(json: &Value) -> anyhow::Result<u64> {
    let mut hasher = DefaultHasher::new();
    match json {
        Value::Object(o) => generate_keys(o, vec![]).hash(&mut hasher),
        _ => Err(anyhow!("Json object, should be a tree not a single leaf."))?,
    };
    Ok(hasher.finish())
}
fn generate_keys(json: &Map<String, Value>, mut keys: Vec<String>) -> Vec<String> {
    for (k, v) in json {
        keys.push(k.clone());
        let Value::Object(o) = v else {
            continue;
        };
        keys = generate_keys(o, keys);
    }
    keys
}
fn calculate_delta(base_json: &Value, comparison_json: &Value) -> Option<Value> {
    assert_eq!(
        calculate_cfg_hash(base_json).unwrap_or(0),
        calculate_cfg_hash(comparison_json).unwrap_or(0),
        "Key structure must be the same in the jsons."
    );
    match (base_json, comparison_json) {
        (Value::Object(base_object), Value::Object(comparsion_object)) => {
            let mut delta = Map::new();
            for (k, base_value) in base_object {
                let comparison_value = comparsion_object.get(k).unwrap();
                if let Some(r_delta) = match (base_value, comparison_value) {
                    (Value::Object(_), Value::Object(_)) => {
                        calculate_delta(base_value, comparison_value)
                    }
                    _ => None,
                } {
                    delta.insert(k.clone(), r_delta);
                } else if comparison_value != base_value {
                    delta.insert(k.clone(), comparison_value.clone());
                };
            }
            if !delta.is_empty() {
                return Some(Value::Object(delta));
            }
            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod test_delta {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_hash_value_invariant() {
        let json = json!({"test": {"really": {"super": 0}, "deep": 0}});
        let json_2 = json!({"test": {"really": {"super": 1}, "deep": 1}});
        let hash = calculate_cfg_hash(&json).unwrap();
        let hash_2 = calculate_cfg_hash(&json_2).unwrap();
        assert_eq!(hash, hash_2);
    }
    #[test]
    fn test_hash_differs_on_key() {
        let json = json!({"test": {"really": {"super": 0}, "deep": 0}});
        let json_2 = json!({"test": {"really": {"super": 0}, "deeper": 0}});
        let hash = calculate_cfg_hash(&json).unwrap();
        let hash_2 = calculate_cfg_hash(&json_2).unwrap();
        assert_ne!(hash, hash_2);
    }
    #[test]
    fn test_hash_fail_on_leaf() {
        let json = json!("string");
        assert!(calculate_cfg_hash(&json).is_err());
    }
    #[test]
    fn test_calculate_delta() {
        let json = json!({"test": {"really": {"super": 0}, "deep": 0}});
        let json_2 = json!({"test": {"really": {"super": 1}, "deep": 1}});
        assert!(calculate_delta(&json, &json_2).is_some())
    }
    #[test]
    fn test_calculate_delta_none() {
        let json = json!({"test": {"really": {"super": 0}, "deep": 0}});
        assert!(calculate_delta(&json, &json).is_none())
    }
    #[test]
    fn test_calculate_delta_structure() {
        let json = json!({"test": {"really": {"super": 0}, "deep": 0}});
        let json_2 = json!({"test": {"really": {"super": 1}, "deep": 1}});
        assert_eq!(calculate_delta(&json, &json_2).unwrap(), json_2)
    }
    #[test]
    fn test_calculate_delta_structure_drops_consistent_keys() {
        let json = json!({"test": {"really": {"super": 0, "duper": 0}, "deep": 0}});
        let json_2 = json!({"test": {"really": {"super": 1, "duper": 0}, "deep": 1}});
        let delta = json!({"test": {"really": {"super": 1}, "deep": 1}});
        assert_eq!(calculate_delta(&json, &json_2).unwrap(), delta);
    }
    #[test]
    fn test_delta_reconstruction() {
        let json = json!({"test": {"really": {"super": 0, "duper": 0}, "deep": 0}});
        let json_2 = json!({"test": {"really": {"super": 1, "duper": 0}, "deep": 1}});
        let delta = calculate_delta(&json, &json_2).unwrap();
        assert_eq!(json_2, build_cfg_from_base_and_delta(json, delta))
    }
}

#[cfg(test)]
mod db_tests {
    use super::*;
    use serde_json::json;
    fn mock_db() -> Store {
        let s = Store::new("sqlite::memory:").unwrap();
        s.block_on(sqlx::migrate!().run(&s.pool)).unwrap();
        s
    }
    #[test]
    fn test_add() {
        let s = mock_db();
        let json = serde_json::json!({"test": 200});
        s.add_base_config("Test".to_owned(), json).unwrap();
    }
    #[test]
    fn test_add_get() {
        let s = mock_db();
        let json = serde_json::json!({"test": 200});
        s.add_base_config("Test".to_owned(), json.clone()).unwrap();
        let cfg = s.get_base_config("Test", None).unwrap();
        assert_eq!(cfg, json)
    }
    #[test]
    fn test_add_multi() {
        let s = mock_db();
        let mut c: i64 = -1;
        let mut id = 0;
        for i in 0..10 {
            let json = serde_json::json!({format!("test{}",i): 200});
            c = s.add_base_config("test".to_owned(), json.clone()).unwrap();
            id = i;
        }
        assert_eq!(c, id);
    }
    #[test]
    fn test_insert_read() {
        let db = mock_db();
        let json = json!({"test": {"really": {"super": 0, "duper": 0}, "deep": 0}});
        db.add_config("test_ins", json.clone()).unwrap();
        let json_out = db.get_latest_config("test_ins", None).unwrap().unwrap();
        assert_eq!(json, json_out);
    }
    #[test]
    fn test_insert_read_version() {
        let db = mock_db();
        let json = json!({"test": {"really": {"super": 0, "duper": 0}, "deep": 0}});
        let json_2 = json!({"tester": {"really": {"super": 0, "duper": 0}, "deep": 0}});
        db.add_config("test_ins", json.clone()).unwrap();
        db.add_config("test_ins", json_2.clone()).unwrap();
        let json_out = db.get_latest_config("test_ins", Some(1)).unwrap().unwrap();
        assert_eq!(json_2, json_out);
    }
    #[test]
    fn test_read_from_delta() {
        let db = mock_db();
        let json = json!({"test": {"really": {"super": 0, "duper": 0}, "deep": 0}});
        let json_2 = json!({"test": {"really": {"super": 1, "duper": 0}, "deep": 1}});
        db.add_config("test_ins", json.clone()).unwrap();
        db.add_config("test_ins", json_2.clone()).unwrap();
        let json_out = db.get_latest_config("test_ins", None).unwrap().unwrap();
        assert_eq!(json_2, json_out);
    }
    #[test]
    fn test_read_all() {
        let db = mock_db();
        let json = json!({"test": {"really": {"super": 0, "duper": 0}, "deep": 0}});
        let json_2 = json!({"tester": {"really": {"super": 1, "duper": 0}, "deep": 1}});
        db.add_config("test_ins", json.clone()).unwrap();
        db.add_config("test_ins", json_2.clone()).unwrap();
        let cfgs = db.get_base_configs().unwrap();
        assert_eq!(cfgs.len(), 2)
    }
}
