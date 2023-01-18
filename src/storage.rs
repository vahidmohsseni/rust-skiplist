use std::{path::{PathBuf, Path}, io::{BufWriter, self, Write}, fs::{File, OpenOptions}, time::{SystemTime, UNIX_EPOCH}};



pub struct Storage {
    path: PathBuf,
    writer: BufWriter<File>
}


impl Storage {
    pub fn new(path: &Path) -> io::Result<Storage> {

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros();

        let path = Path::new(path).join(format!("{}.db", timestamp.to_string()));

        let file = OpenOptions::new().create(true).append(true).open(&path).unwrap();

        let writer = BufWriter::new(file);
        

        Ok(Storage { 
            path,
            writer 
        })
    }

    pub fn from_path(path: &Path) -> io::Result<Storage> {
        let file = OpenOptions::new().append(true).create(true).open(&path)?;
        let writer = BufWriter::new(file);

        Ok(Storage { 
            path: path.to_path_buf(),
            writer
        })
    }

    // The data layout:
    // +---------------+-------------------+-----------------+----------+------------+-----------------+ 
    // | Key size (8B) | Deleted flag (1B) | Value size (8B) | key (?B) | value (?B) | timestamp (16B) |
    // +---------------+-------------------+-----------------+----------+------------+-----------------+ 
    // 
    pub fn set(&mut self, key: &[u8], value: &[u8], deleted: bool, timestamp: u128) -> io::Result<()>{
        self.writer.write_all(&key.len().to_le_bytes())?;
        self.writer.write_all(&(deleted as u8).to_le_bytes())?;
        self.writer.write_all(&value.len().to_le_bytes())?;

        self.writer.write_all(key)?;
        self.writer.write_all(value)?;

        self.writer.write_all(&timestamp.to_le_bytes())?;

        Ok(())
    }

    pub fn delete(&mut self, key: &[u8], timestamp: u128) -> io::Result<()>{
        self.writer.write_all(&key.len().to_le_bytes())?;
        self.writer.write_all(&(true as u8).to_le_bytes())?;
        let value_size = 0x0000 as u64;
        self.writer.write_all(&value_size.to_le_bytes())?;

        self.writer.write_all(&timestamp.to_le_bytes())?;

        Ok(())
    }

    pub fn commit(&mut self) -> io::Result<()> {
        self.writer.flush()?;
        Ok(())
    }

}


#[cfg(test)]
mod test {

    use std::{path::PathBuf, time::SystemTime, io::Read};
    use rand::Rng;
    use crate::utils::{file_reader, create_dir, scan_dir, remove_dir};

    use super::Storage;

    #[test]
    fn test_create(){
        let mut range = rand::thread_rng();
        let path = PathBuf::from(format!("./temp-{}", range.gen::<u32>()));
        
        create_dir(&path).unwrap();

        let mut storage = Storage::new(&path).unwrap();
        
        let key = b"Hello".to_owned();
        let value = *b"World!";
        let timestamp = SystemTime::now().elapsed().unwrap().as_micros(); 
        storage.set(&key, &value, false, timestamp).expect("Error: could not writer in the file");
        storage.commit().expect("Error in flush!");

        let mut line = [0 as u8; 28];

        let files = scan_dir(&path).expect(&format!("Error: could not scan the dir: {:?}", path));
        let mut reader = file_reader(&files[0]);

        reader.read_exact(&mut line).expect("Error: could not read the file");
        assert_eq!(line[17..], *b"HelloWorld!");

        // Clean up
        remove_dir(&path).expect("Error: could not remove the directory");
    }
}