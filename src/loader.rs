use std::fs::File;
use std::io::prelude::*;

// TODO: try to find maskfile in current directory and maybe parent directories?
// https://github.com/jakedeichert/mask/issues/7

pub fn read_maskfile(maskfile: &str) -> Result<String, String> {
    let file = File::open(maskfile);
    if file.is_err() {
        return Err("failed to open maskfile.md".to_string());
    }

    let mut file = file.unwrap();
    let mut maskfile_contents = String::new();
    file.read_to_string(&mut maskfile_contents)
        .expect("expected file contents");

    Ok(maskfile_contents)
}

#[cfg(test)]
mod read_maskfile {
    use super::*;

    #[test]
    fn reads_root_maskfile() {
        let maskfile = read_maskfile("./maskfile.md");

        assert!(maskfile.is_ok(), "maskfile was ok");

        let contents = maskfile.unwrap();

        // Basic test to make sure the maskfile.md contents are at least right
        let expected_root_description = "> Development tasks for mask";
        assert!(
            contents.contains(expected_root_description),
            "description wasn't found in maskfile contents"
        );
    }

    #[test]
    fn errors_for_non_existent_maskfile() {
        let maskfile = read_maskfile("src/maskfile.md");

        assert!(maskfile.is_err(), "maskfile was err");

        let err = maskfile.unwrap_err();

        let expected_err = "failed to open maskfile.md";
        assert_eq!(err, expected_err, "error message was wrong");
    }
}
