use mime_guess::MimeGuess;
use std::env;
use std::path::Path;

pub fn get_file_contents_with_mime(file: &String) -> Result<(Vec<u8>, MimeGuess), String> {
    let static_dir = match env::var("STATIC_DIR") {
        Ok(val) => val,
        _ => String::from("static"),
    };

    let path = Path::new(&static_dir[..]).join(&file.clone().to_owned()[..]);
    log::info!("Searching for {:?}", path);
    if !path.exists() || !path.is_file() {
        return Err(String::from("Not found"));
    }

    let data = std::fs::read(&path).map_err(|e| e.to_string())?;

    let filename: &str = &file.clone().to_owned()[..];
    return Ok((data, mime_guess::from_path(filename)));
}