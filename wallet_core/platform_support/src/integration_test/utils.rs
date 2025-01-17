use tokio::fs::{self, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::utils::PlatformUtilities;

// This utility function is used both by the Rust integration test for the "software" feature
// and by integration test performed from Android / iOS.
// This would normally fall under dev-dependencies, however we need it in the main binary
// for the Android / iOS integration test.
pub async fn get_and_verify_storage_path<K: PlatformUtilities>() -> bool {
    let original_message = "Hello, wallet!";
    let mut path = K::storage_path()
        .await
        .expect("Could not get storage path")
        .into_os_string()
        .into_string()
        .expect("Could not convert PathBuf to String");

    // Perform basic path sanity check
    assert!(!path.is_empty());
    assert!(path.starts_with('/'));

    // Create the test.txt file path
    path.push_str("/test.txt");

    // Write the [original_message] to test.txt
    let mut test_file = File::create(&path).await.expect("Could not create test.txt file");
    test_file
        .write_all(original_message.as_bytes())
        .await
        .expect("Could not write to file");
    test_file.flush().await.expect("Could not flush file");

    // Open the test.txt file and read the contents
    let mut test_file_ro = File::open(&path).await.expect("Could not open test.txt file");
    let mut contents = String::new();
    test_file_ro
        .read_to_string(&mut contents)
        .await
        .expect("Could not read test.txt");

    // Clean up and verify the file contents match the [original_message]
    fs::remove_file(&path).await.expect("Could not delete test.txt");

    contents == original_message
}

#[cfg(feature = "hardware-integration-test")]
mod hardware {
    use jni::{objects::JClass, JNIEnv};

    use super::get_and_verify_storage_path;
    use crate::utils::hardware::HardwareUtilities;

    // this is the starting point for the integration test performed from Android / iOS.
    #[no_mangle]
    fn utils_test_get_storage_path() -> bool {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(get_and_verify_storage_path::<HardwareUtilities>())
    }

    #[no_mangle]
    extern "C" fn Java_nl_rijksoverheid_edi_wallet_platform_1support_utilities_UtilitiesBridgeInstrumentedTest_utilities_1test_1storage_1path(
        _env: JNIEnv,
        _: JClass,
    ) -> bool {
        utils_test_get_storage_path()
    }
}
