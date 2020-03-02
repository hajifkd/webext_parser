use std::io;
use std::path::Path;
use tokio::fs;

pub(crate) async fn get_cached(url: &str) -> io::Result<String> {
    let cache_dir = Path::new("cache");

    if cache_dir.exists() {
        if !cache_dir.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                "File exists",
            ));
        }
    } else {
        fs::create_dir(&cache_dir).await?;
    }

    let cache_file = cache_dir.join(url.replace("/", "_").replace(":", "_"));

    if cache_file.exists() {
        fs::read_to_string(&cache_file).await
    } else {
        let html = reqwest::get(url)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::AddrNotAvailable, format!("{}", e)))?
            .text()
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{}", e)))?;
        fs::write(&cache_file, &html).await?;
        Ok(html)
    }
}

pub(crate) enum TakeResult<T> {
    Zero,
    One(T),
    More,
}

pub(crate) fn take_one<T>(iter: impl Iterator<Item = T>) -> TakeResult<T> {
    let mut v = iter.collect::<Vec<_>>();

    if v.len() == 0 {
        TakeResult::Zero
    } else if v.len() == 1 {
        TakeResult::One(v.remove(0))
    } else {
        TakeResult::More
    }
}
