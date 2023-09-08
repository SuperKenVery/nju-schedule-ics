// Use redis with its persistence feature as the db

use redis::{Client,AsyncCommands};

pub struct RedisDb{
    connection: redis::aio::Connection,
}

impl RedisDb {
    pub async fn new(url: &str) -> Result<Self,anyhow::Error> {
        let client=Client::open(url)?;
        let connection=client.get_async_connection().await?;

        Ok(RedisDb { connection })
    }

    pub async fn insert<K,V>(&mut self, key: K, value: V) -> Result<(),anyhow::Error>
    where
        K: ToString,
        V: ToString,
    {
        let key=key.to_string();
        let value=value.to_string();
        println!("Insert {} {}", key, value);
        self.connection.set(key, value).await?;


        Ok(())
    }

    pub async fn get<K>(&mut self, key: K) -> Result<Option<String>,anyhow::Error>
    where
        K: ToString,
    {
        let key=key.to_string();
        println!("Getting {}", key);
        let value=self.connection.get(key).await?;

        Ok(value)
    }


}
