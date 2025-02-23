use super::*;


#[derive(Debug)]
pub struct StorageFile {
    pub directory: String
}

#[async_trait]
impl ComponentInteraction for StorageFile {
    async fn store(&self, table_def: TableDef) -> Result<(), StorageError> {
        let filename = format!("{}/{}.json", self.directory, table_def.info.name);
        let path = Path::new(&filename);
        let file = File::create(path).map_err(|_err| error::StorageError::Unhandled)?;

        serde_json::to_writer(file, &table_def).expect("to write json file");
        Ok(())
    }
    async fn store_component(&self, component: Component) -> Result<(), error::StorageError>{
        todo!("store component")
    }

    async fn fetch(&self, table_name: &str) -> Result<TableDef, StorageError> {
        let filename = format!("{}/{}.json", self.directory, table_name);
        let path = Path::new(&filename);
        let file = File::open(path).map_err(|_err| StorageError::Unhandled)?;

        let content: TableDef= serde_json::from_reader(file).expect("to read file");
        Ok(content)
    }

    async fn index(&self) -> Result<Vec<String>, StorageError> {
        let path = Path::new(&self.directory);

        let files: Vec<String> = fs::read_dir(&path)
            .map_err(|_err| StorageError::Unhandled)?
            .into_iter()
            .filter_map(|f| f.ok())
            .map(|f| format!("{:?}", f.file_name()))
            .collect();

        Ok(files)
    }
}
