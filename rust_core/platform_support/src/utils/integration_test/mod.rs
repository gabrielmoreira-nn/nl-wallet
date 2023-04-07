use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use crate::utils::PlatformUtilities;

#[cfg(all(feature = "hardware-integration-test"))]
pub mod hardware;

pub fn get_and_verify_storage_path<K: PlatformUtilities>() -> bool {
    let original_message = "Hello, wallet!";
    let path = K::storage_path().expect("Could not get storage path")
        .into_os_string().into_string().expect("Could not convert PathBuf to String");

    // Perform basic path sanity check
    assert!(path.len() > 0);
    assert!(path.starts_with("/"));

    // Create the test.txt file path
    let mut test_file_path = path.to_owned();
    test_file_path.push_str("/test.txt");

    // Write the [original_message] to test.txt
    let mut test_file = File::create(&test_file_path).expect("Could not create test.txt file");
    test_file.write_all(original_message.as_bytes()).expect("Could not write to file");
    test_file.flush().expect("Could not flush file");

    // Open the test.txt file and read the contents
    let mut test_file_ro = File::open(&test_file_path).expect("Could not open test.txt file");
    let mut contents = String::new();
    test_file_ro.read_to_string(&mut contents).expect("Could not read test.txt");

    // Verify the file contents match the [original_message] and clean up
    assert_eq!(contents, original_message);
    fs::remove_file(&test_file_path).expect("Could not delete test.txt");

    true
}
