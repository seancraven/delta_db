use anyhow::anyhow;
use delta_backend::{read_file, Store};
use pyo3::{pyclass, pymethods, pymodule, types::PyModule, Bound, PyObject, PyResult};
use std::path::PathBuf;
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
