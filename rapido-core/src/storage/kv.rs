use std::fmt::Debug;

use super::*;

pub struct StorageKv {
    items: PartitionHandle,
}
impl Debug for StorageKv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let p = self.items.path().display();
        f.write_str("kv")
    }
}

impl StorageKv {
    pub fn new() -> Result<Self, StorageError> {
        let keyspace = fjall::Config::new("./fjall")
            .open()
            .map_err(|err| StorageError::Unhandled)?;
        let items = keyspace
            .open_partition("components", PartitionCreateOptions::default())
            .map_err(|err| StorageError::Unhandled)?;

        Ok(Self { items })
    }
}

#[async_trait]
impl ComponentInteraction for StorageKv {
    async fn store(&self, table_def: TableDef) -> Result<(), StorageError> {
        let s = serde_json::to_vec(&table_def).expect("seialize table");
        let table_name = format!("table_{}", table_def.info.name);
        Ok(self.items.insert(table_name, &s)?)
    }

    async fn store_component(&self, component: Component) -> Result<(), StorageError> {
        let component_name = component.component_name();
        let value = serde_json::to_vec(&component)?;
        Ok(self.items.insert(component_name, value)?)
    }
    async fn fetch(&self, table_name: &str) -> Result<TableDef, StorageError> {
        let value = self.items.get(table_name)?.ok_or(StorageError::NotFound)?;
        Ok(serde_json::from_slice(&value)?)
    }

    async fn fetch_component(&self, name: &str) -> Result<Component, StorageError> {
        let value = self.items.get(name)?.ok_or(StorageError::NotFound)?;
        Ok(serde_json::from_slice(&value)?)
    }

    async fn index(&self) -> Result<Vec<String>, StorageError> {
        let mut vs = Vec::new();
        for kv in self
            .items
            .prefix::<&str>("table_")
            .filter_map(|x| x.ok())
            .map(|(k, v)| k)
        {
            if let Some(s) = kv.strip_prefix("table_".as_bytes()) {
                let x = std::str::from_utf8(s)
                    .map(String::from)
                    .expect("string from prefix");
                vs.push(x);
            }
        }
        Ok(vs)
    }

    async fn index_component(&self) -> Result<Vec<String>, StorageError> {
        let mut vs = Vec::with_capacity(self.items.len()?);
        for (key, _value) in self.items.prefix("component").filter_map(|x| x.ok()) {
            vs.push(String::from_utf8(key.to_vec()).map_err(|_| StorageError::Unhandled)?);
        }
        Ok(vs)
    }
}
