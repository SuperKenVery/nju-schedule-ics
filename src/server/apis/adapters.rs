use anyhow::Result;
use dioxus::prelude::*;

#[cfg(feature = "server")]
mod server {
    use crate::adapters::{nju_batchelor::NJUBatchelorAdaptor, traits::School};
    use diesel::SqliteConnection;
    use std::sync::{Arc, Mutex};
    use std::{collections::HashMap, sync::LazyLock};

    pub type SchoolFactory = fn(Arc<Mutex<SqliteConnection>>) -> Box<dyn School>;
    const ADAPTERS: LazyLock<HashMap<&str, SchoolFactory>> = LazyLock::new(|| {
        let mut map = HashMap::<&str, SchoolFactory>::new();
        map.insert("南京大学本科生", |db| {
            Box::new(NJUBatchelorAdaptor::new(db)) as Box<dyn School>
        });
        map.insert("test", |db| todo!());
        map.insert("test2", |db| todo!());

        map
    });

    pub fn adapters() -> Vec<&'static str> {
        ADAPTERS.keys().copied().collect()
    }
}

#[server]
pub async fn available_adapters() -> Result<Vec<String>, ServerFnError> {
    Ok(server::adapters().iter().map(|x| x.to_string()).collect())
}

#[server]
pub async fn set_adapter(name: String) -> Result<(), ServerFnError> {
    todo!()
}
