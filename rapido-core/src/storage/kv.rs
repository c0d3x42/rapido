use super::*;

pub struct StorageKv {
    items: PartitionHandle,
}

impl StorageKv {
    fn new() -> Result<Self, StorageError> {
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
    async fn save(&self, table_def: TableDef) -> Result<(), StorageError> {
        let s = serde_json::to_vec(&table_def).expect("seialize table");
        let table_name = format!("table_{}", table_def.info.name);
        self.items
            .insert(table_name, &s)
            .map_err(StorageError::from)
    }
    async fn load(&self, table_name: &str) -> Result<TableDef, StorageError> {
        let x = self.items.get(table_name).map_err(StorageError::from)?;

        if let Some(slice) = x {
            let table_def: TableDef = serde_json::from_slice(&slice)?;
            Ok(table_def)
        } else {
            Err(StorageError::NotFound)
        }
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
}
