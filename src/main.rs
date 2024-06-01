use std::env;
use std::error;
use std::fs;
use std::fmt;
use std::io;
use flate2::read::ZlibDecoder;

#[derive(Debug, PartialEq, Eq)]
enum Error {
    MalformedObject,
    ObjectNotFound,
    InvalidObjectHash(String),
    UnknownCommand(String),

}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Error::MalformedObject => write!(f, "malformed git object"),
            Error::ObjectNotFound => write!(f, "git object not found"),
            Error::InvalidObjectHash(hash) => write!(f, "invalid object hash: {hash}"),
            Error::UnknownCommand(command) => write!(f, "unknown command: {command}"),
        }
    }
}

impl error::Error for Error {}

fn init() -> Result<(), io::Error> {
    fs::create_dir(".git")?;
    fs::create_dir(".git/objects")?;
    fs::create_dir(".git/refs")?;
    fs::write(".git/HEAD", "ref: refs/heads/main\n")?;
    println!("Initialized git directory");
    Ok(())
}

fn unknown_command(command: &str) -> Result<(), Error> {
    Err(Error::UnknownCommand(command.to_string()))
}

fn parse_blob<R: io::Read>(blob: Option<R>) -> Result<String, Error> {
    let mut blob = blob.ok_or(Error::ObjectNotFound)?;
    let mut header: [u8; 4] = [0; 4];
    blob.read_exact(&mut header).map_err(|_|Error::MalformedObject)?;
    let header = String::from_utf8(header.to_vec()).unwrap();
    if header != "blob" {
        return Err(Error::MalformedObject);
    }

    let mut content = String::new();
    blob.read_to_string(&mut content).map_err(|_|Error::MalformedObject)?;
    let (size, content) = content.trim_start().split_once('\0').ok_or(Error::MalformedObject)?;
    if size.to_string().parse::<u8>().is_err() {
        return Err(Error::MalformedObject);
    }
    Ok(content.to_string())
}

fn parse_object_path_from_hash(hash: &str) -> Result<String, Error> {
    if hash.chars().count() == 40 && hash.chars().all(char::is_alphanumeric) {
        let (dir, filename) = hash.split_at(2);
        return Ok(format!("./.git/objects/{dir}/{filename}"));
    }
    Err(Error::InvalidObjectHash(hash.to_string()))
}

fn cat_file(hash: &str) -> Result<(), Error> {
    let path = parse_object_path_from_hash(hash)?;
    let blob = fs::File::open(path).map(ZlibDecoder::new).ok();
    let content = parse_blob(blob)?;
    print!("{content}");
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Err(err)  = match args[1].as_str() {
        "init" => init().map_err(|err|Box::new(err) as Box<dyn error::Error>),
        "cat-file" => cat_file(&args[3]).map_err(|err|Box::new(err) as Box<dyn error::Error>),
        _ => unknown_command(&args[1]).map_err(|err|Box::new(err) as Box<dyn error::Error>)
    } {
        println!("{err}");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_blob_succeeds_with_good_blob() -> Result<(), Error> {
        let blob_content = "blob 7\0abcd123".as_bytes();
        let expected_content = "abcd123";

        let content = parse_blob(Some(blob_content))?;
        assert_eq!(content, expected_content);
        Ok(())
    }

    #[test]
    fn parse_blob_is_error_with_bad_blob() {
        let blob_content = "12345".as_bytes();
        let expected_error = Err(Error::MalformedObject);

        let actual_result= parse_blob(Some(blob_content));
        assert_eq!(actual_result, expected_error);
    }

    #[test]
    fn parse_blob_is_error_with_no_blob_header() {
        let blob_content = "7\0abcd123".as_bytes();
        let expected_error = Err(Error::MalformedObject);

        let actual_result = parse_blob(Some(blob_content));
        assert_eq!(actual_result, expected_error);
    }

    #[test]
    fn parse_blob_is_error_with_no_size() {
        let blob_content = "blob \0abcd123".as_bytes();
        let expected_error = Err(Error::MalformedObject);

        let actual_result = parse_blob(Some(blob_content));
        assert_eq!(actual_result, expected_error);
    }

    #[test]
    fn parse_blob_is_error_with_nonexistent_blob() {
        let blob_content: Option<&[u8]> = None;
        let expected_error = Err(Error::ObjectNotFound);

        let actual_result = parse_blob(blob_content);
        assert_eq!(actual_result, expected_error);
    }

    #[test]
    fn test_unknown_command() {
        let command = "unknown-command";
        let expected_error = Err(Error::UnknownCommand(command.to_string()));

        let actual_result = unknown_command(command);
        assert_eq!(actual_result, expected_error);
    }

    #[test]
    fn test_parse_object_path_from_hash() -> Result<(), Error> {
        let object_hash = "a1b2c3d4e5f6g7h8i9j0a1b2c3d4e5f6g7h8i9j0";
        let expected_path = "./.git/objects/a1/b2c3d4e5f6g7h8i9j0a1b2c3d4e5f6g7h8i9j0";

        let path = parse_object_path_from_hash(object_hash)?;
        assert_eq!(path, expected_path);
        Ok(())
    }
    
    #[test]
    fn test_parse_object_path_from_hash_with_invalid_chars() {
        let object_hash = "a1b2c3d4e5f6g7h8i9j/a1b2c3d4e5f6g7h8i9j/";
        let expected_error = Err(Error::InvalidObjectHash(object_hash.to_string()));

        let actual_result = parse_object_path_from_hash(object_hash);
        println!("{:?}", actual_result);
        assert_eq!(actual_result, expected_error);
    }

    #[test]
    fn test_parse_object_path_from_hash_with_short_length() {
        let object_hash = "1234567890";
        let expected_error = Err(Error::InvalidObjectHash(object_hash.to_string()));

        let actual_result = parse_object_path_from_hash(object_hash);
        assert_eq!(actual_result, expected_error);
    }

    #[test]
    fn test_parse_object_path_from_hash_with_long_length() {
        let object_hash = "a1b2c3d4e5f6g7h8i9j0a1b2c3d4e5f6g7h8i9j01234567890";
        let expected_error = Err(Error::InvalidObjectHash(object_hash.to_string()));

        let actual_result = parse_object_path_from_hash(object_hash);
        assert_eq!(actual_result, expected_error);
    }
}

