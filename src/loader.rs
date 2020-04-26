use std::fs::File;
use std::io::prelude::*;

pub fn read_maskfile(maskfile: &str) -> (Result<String, String>, String) {
    let mut filename = String::from(maskfile);
    if filename == "" {
        let p = std::env::current_dir().unwrap();
        for ancestor in p.ancestors() {
            let check = ancestor.join("orders.md");
            let file = File::open(&filename);
            if file.is_ok() {
                filename = String::from(check.to_str().unwrap());
                return (Ok(read_and_return(file)), filename);
            }
        }
        return (
            Err("Could not locate an orders.md file".to_owned()),
            filename,
        );
    }
    let file = File::open(&filename);
    if file.is_err() {
        return (Err(format!("failed to open {}", filename)), filename);
    }
    let maskfile_contents = read_and_return(file);
    (Ok(maskfile_contents), filename)
}

fn read_and_return(file: Result<std::fs::File, std::io::Error>) -> String {
    let mut file = file.unwrap();
    let mut maskfile_contents = String::new();
    file.read_to_string(&mut maskfile_contents)
        .expect("expected file contents");
    maskfile_contents
}

#[cfg(test)]
mod read_maskfile {
    use super::*;

    #[test]
    fn reads_root_maskfile() {
        let (maskfile, _) = read_maskfile("./maskfile.md");

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
        let (maskfile, _) = read_maskfile("src/maskfile.md");

        assert!(maskfile.is_err(), "maskfile was err");

        let err = maskfile.unwrap_err();

        let expected_err = "failed to open maskfile.md";
        assert_eq!(err, expected_err, "error message was wrong");
    }
}
