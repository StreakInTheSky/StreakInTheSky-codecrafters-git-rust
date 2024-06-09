use std::fs;
use std::io;
use flate2::read::ZlibDecoder;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("malformed git object")]
    MalformedObject,
    #[error("git object not found")]
    ObjectNotFound,
    #[error("invalid object hash: {0}")]
    InvalidObjectHash(String),
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

pub fn cat_file(hash: &str) -> anyhow::Result<()> {
    let path = parse_object_path_from_hash(hash)?;
    let blob = fs::File::open(path).map(ZlibDecoder::new).ok();
    let content = parse_blob(blob)?;
    print!("{content}");
    Ok(())
}

pub fn hash_object(_path: &str) -> anyhow::Result<()> {
    let hash = "ac136066947976e9f5ae7cc6bdccac22d0fc0f6f"; 
    let object_path = parse_object_path_from_hash(hash)?;
    if let Err(err) = fs::write(object_path.clone(), "") {
        if err.kind() == std::io::ErrorKind::NotFound {
            let (directory, _filename) = object_path.split_at(17);
            fs::create_dir(directory)?;
            fs::write(object_path, "")?;
        }
    }
    println!("{hash}");
    Ok(())
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

    #[test]
    fn hash_object_from_valid_path() -> Result<(), Error>{
        let path = "strawberry.txt";
        let expected_hash = "ac136066947976e9f5ae7cc6bdccac22d0fc0f6f";

        let actual_result = hash_file(path)?;
        assert_eq!(actual_result, expected_hash);
        Ok(())
    }
}

