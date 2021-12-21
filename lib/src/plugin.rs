use std::collections::HashMap;
use std::fmt;

pub fn indradb_version_info() -> VersionInfo {
    VersionInfo {
        rustc: env!("RUSTC_VERSION").to_string(),
        indradb_interface: 0,
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct VersionInfo {
    pub rustc: String,
    pub indradb_interface: u8,
}

impl fmt::Display for VersionInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rustc={}, indradb_interface={}", self.rustc, self.indradb_interface)
    }
}

pub trait Plugin: 'static + Send + Sync {
    fn call(&self, datastore: Box<dyn crate::Transaction>, arg: serde_json::Value) -> crate::Result<serde_json::Value>;
}

pub struct PluginDeclaration {
    pub version_info: VersionInfo,
    pub entries: HashMap<String, Box<dyn Plugin>>,
}

#[macro_export]
macro_rules! register_plugins {
    ( $indradb_interface_version:expr, $( $name:expr, $t:expr ),* ) => {
        use indradb::plugin::PluginDeclaration;
        #[doc(hidden)]
        #[no_mangle]
        pub unsafe extern "C" fn register() -> indradb::plugin::PluginDeclaration {
            use std::collections::HashMap;
            let mut entries = HashMap::new();
            $(
                {
                    let t: Box<dyn indradb::plugin::Plugin> = $t;
                    entries.insert($name.to_string(), t);
                }
            )*
            PluginDeclaration {
                version_info: indradb::plugin::VersionInfo {
                    // TODO: ensure env! executes at macro expansion time
                    rustc: env!("RUSTC_VERSION").to_string(),
                    indradb_interface: $indradb_interface_version,
                },
                entries,
            }
        }
    };
}
