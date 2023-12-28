use std::io::Read;

use flate2::read::ZlibDecoder;

pub fn get_object_directory_name(hash: &str) -> String {
    hash[0..2].to_owned()
}

pub fn get_object_file_name(hash: &str) -> String {
    hash[2..].to_owned()
}

pub fn decompress(bytes: &[u8]) -> Vec<u8> {
    let mut decoder = ZlibDecoder::new(bytes);
    let mut result = vec![];

    decoder.read_to_end(&mut result).unwrap();

    result
}

pub fn index_of_next_null(bytes: &[u8], offset_index: usize) -> Option<usize> {
    if let Some(index) = bytes
        .iter()
        .skip(offset_index)
        .position(|&byte| byte == b'\0')
    {
        Some(index + offset_index)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use flate2::{write::ZlibEncoder, Compression};

    use super::*;

    #[test]
    fn should_get_object_directory_name_from_hash() {
        let hash = "8515244e62a6f01fea3d4866a4e075782b81a05e";
        let expected_name = "85";
        let name = get_object_directory_name(hash);

        assert_eq!(name, expected_name);
    }

    #[test]
    fn should_get_object_file_name() {
        let hash = "8515244e62a6f01fea3d4866a4e075782b81a05e";
        let expected_name = "15244e62a6f01fea3d4866a4e075782b81a05e";
        let name = get_object_file_name(hash);

        assert_eq!(name, expected_name);
    }

    #[test]
    fn should_decompress() {
        let de_compressed_string = "8515244e62a6f01fea3d4866a4e075782b81a05e";
        let mut encoder = ZlibEncoder::new(vec![], Compression::default());

        encoder.write_all(de_compressed_string.as_bytes()).unwrap();

        let compressed = encoder.finish().unwrap();
        let de_compressed = decompress(&compressed);

        assert_eq!(de_compressed, de_compressed_string.as_bytes());
    }

    #[test]
    fn should_return_index_of_next_null() {
        let string = "eanfphensrtduyfj\0rsiueaptyrafupgdreif\0";
        let expected_index = 16;
        let result = index_of_next_null(string.as_bytes(), 0).unwrap();

        assert_eq!(result, expected_index);
    }

    #[test]
    fn should_return_index_of_second_null() {
        let string = "eanfphensrtduyfj\0rsiueaptyrafupgdreif\0aoiresth";
        let string_bytes = string.as_bytes();
        let first_index = index_of_next_null(string_bytes, 0).unwrap();
        let expected_index = 37;
        let result = index_of_next_null(string_bytes, first_index + 1).unwrap();

        assert_eq!(result, expected_index);
    }
}
