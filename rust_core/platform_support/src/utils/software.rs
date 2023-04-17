use std::{env::temp_dir, path::PathBuf};

use super::{PlatformUtilities, UtilitiesError};

pub struct SoftwareUtilities;

impl PlatformUtilities for SoftwareUtilities {
    fn storage_path() -> Result<PathBuf, UtilitiesError> {
        let path = temp_dir();

        Ok(path)
    }
}