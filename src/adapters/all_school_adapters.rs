use crate::adapters::{nju_batchelor::NJUBatchelorAdaptor, traits::School};
use diesel::SqliteConnection;
use std::{collections::HashMap, sync::LazyLock};
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::Mutex;

pub type SchoolFactory =
    fn(Arc<Mutex<SqliteConnection>>) -> Pin<Box<dyn Future<Output = Box<dyn School>> + Send>>;
const SCHOOL_ADAPTERS: LazyLock<HashMap<&str, SchoolFactory>> = LazyLock::new(|| {
    let mut map = HashMap::<&str, SchoolFactory>::new();
    map.insert("南京大学本科生", |db| {
        Box::pin(async { Box::new(NJUBatchelorAdaptor::new(db).await) as Box<dyn School> })
    });
    map.insert("test", |db| todo!());
    map.insert("test2", |db| todo!());

    map
});

pub fn school_adapters() -> Vec<&'static str> {
    SCHOOL_ADAPTERS.keys().copied().collect()
}

pub async fn school_adapter_from_name(
    name: &str,
    db: Arc<Mutex<SqliteConnection>>,
) -> Option<Box<dyn School>> {
    // let builder = SCHOOL_ADAPTERS.get(name)?;
    Some((SCHOOL_ADAPTERS.get(name)?)(db).await)
}
