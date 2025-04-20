use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{self, BufReader, BufWriter, Write},
    path::Path,
};

pub fn read<T, P>(path: P) -> std::io::Result<(T)>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let value = serde_json::from_reader(reader)?;
    Ok(value)
}

pub fn write<T, P>(path: P, value: &T) -> std::io::Result<()>
where
    T: Serialize,
    P: AsRef<Path>,
{
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, value)?;
    Ok(())
}

pub fn append_json_to_array<T, P>(path: P, item: &T) -> io::Result<()>
where
    T: Serialize + DeserializeOwned + Clone,
    P: AsRef<Path>,
{
    let mut vec: Vec<T> = match read(&path) {
        Ok(existing) => existing,
        Err(e) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err(e),
    };

    vec.push(item.clone());

    write(path, &vec)
}
