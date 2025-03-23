use std::error::Error;
use hemtt_sqm::SqmFile;

/// Parse SQM content and return a SqmFile structure from HEMTT's sqm library
pub(crate) fn parse_sqm_content(content: &str) -> Result<SqmFile, String> {
    match hemtt_sqm::parse_sqm(content) {
        Ok(sqm_file) => Ok(sqm_file),
        Err(_) => Err("Failed to parse SQM content".into())
    }
}