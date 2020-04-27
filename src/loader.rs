use std::fs::File;
use std::io::prelude::*;

pub fn read_inkfile(inkfile: &str) -> (Result<String, String>, String) {
    let mut filename = String::from(inkfile);
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
    let inkfile_contents = read_and_return(file);
    (Ok(inkfile_contents), filename)
}

fn read_and_return(file: Result<std::fs::File, std::io::Error>) -> String {
    let mut file = file.unwrap();
    let mut inkfile_contents = String::new();
    file.read_to_string(&mut inkfile_contents)
        .expect("expected file contents");
    inkfile_contents
}

#[cfg(test)]
mod read_inkfile {
    use super::*;

    #[test]
    fn reads_root_inkfile() {
        let (inkfile, _) = read_inkfile("./inkjet.md");

        assert!(inkfile.is_ok(), "inkfile was ok");

        let contents = inkfile.unwrap();

        // Basic test to make sure the inkjet.md contents are at least right
        let expected_root_description = "> Development tasks for inkjet";
        assert!(
            contents.contains(expected_root_description),
            "description wasn't found in inkfile contents"
        );
    }

    #[test]
    fn errors_for_non_existent_inkfile() {
        let (inkfile, _) = read_inkfile("src/inkjet.md");

        assert!(inkfile.is_err(), "inkfile was err");

        let err = inkfile.unwrap_err();

        let expected_err = "failed to open src/inkjet.md";
        assert_eq!(err, expected_err, "error message was wrong");
    }
}
