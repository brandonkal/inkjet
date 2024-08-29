use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;

/// Reads process standard input to a String
pub fn read_stdin() -> Result<String, String> {
    let mut buffer = String::new();
    let r = io::stdin().read_to_string(&mut buffer);
    if r.is_err() {
        Err("failed to read stdin".to_string()) // cov:ignore (unusual)
    } else {
        Ok(buffer)
    }
}
/// Build a fake filename for display in "generated by" help text when stdin is read as the source
fn stdin_name() -> String {
    let pwd = std::env::current_dir().expect("os did not supply working directory");
    String::from(
        pwd.join("stdin")
            .to_str()
            .expect("could not convert stdin path to string"),
    )
}

/// reads an inkfile. If the input contains multiple lines,
/// it is parsed as the text contents.
/// If it does not, it is assumed to be a filename.
/// returns contents, filename, and if it was a real file
pub fn read_inkfile(inkfile: &str) -> (Result<String, String>, String, bool) {
    if inkfile.contains('\n') || inkfile.contains("\r\n") {
        return (Ok(String::from(inkfile)), stdin_name(), false);
    }
    let mut filename = String::from(inkfile);
    if filename == "-" {
        let contents = read_stdin();
        return (contents, stdin_name(), false);
    }
    if filename.is_empty() {
        let p = std::env::current_dir().expect("cannot determine current directory");
        for ancestor in p.ancestors() {
            let check = ancestor.join("inkjet.md");
            let file = File::open(&check);
            if file.is_ok() {
                filename = String::from(check.to_str().unwrap());
                return (Ok(read_and_return(file)), filename, true);
            }
        }
        return (
            Err("Could not locate an inkjet.md file".to_string()),
            filename,
            true,
        );
    }
    let file = File::open(&filename);
    if file.is_err() {
        return (Err(format!("failed to open {}", filename)), filename, true);
    }
    let inkfile_contents = read_and_return(file);
    (Ok(inkfile_contents), filename, true)
}

fn read_and_return(file: Result<std::fs::File, std::io::Error>) -> String {
    let mut file = file.unwrap();
    let mut inkfile_contents = String::new();
    file.read_to_string(&mut inkfile_contents)
        .expect("expected file contents");
    inkfile_contents
}

/// Finds an inkfile and returns its contents and inkfile_path
pub fn find_inkfile(inkfile_opt: &str) -> (Result<String, String>, String) {
    let (contents, inkfile_path, is_file) = read_inkfile(inkfile_opt);
    if contents.is_err() {
        (contents, "".to_string())
    } else if is_file {
        // Find the absolute path to the inkfile
        let absolute_path = fs::canonicalize(&inkfile_path)
            .expect("canonicalize inkfile path failed")
            .to_str()
            .expect("path contained invalid UTF-8 characters")
            .to_string();

        (contents, absolute_path)
    } else {
        (contents, inkfile_path)
    }
}

#[cfg(test)]
mod read_inkfile {
    use super::*;

    #[test]
    fn reads_root_inkfile() {
        let (inkfile, _, _) = read_inkfile("./inkjet.md");

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
        let (inkfile, _, _) = read_inkfile("src/inkjet.md");

        assert!(inkfile.is_err(), "inkfile was err");

        let err = inkfile.unwrap_err();

        let expected_err = "failed to open src/inkjet.md";
        assert_eq!(err, expected_err, "error message was wrong");
    }

    #[test]
    fn stdin_name_works() {
        let sn = stdin_name();
        assert!(sn.contains("stdin"))
    }

    #[test]
    fn reads_stdin() {
        let (_inkfile, inkfile_path, is_file) = read_inkfile("");
        assert!(inkfile_path.contains("inkjet.md"));
        assert!(is_file);
    }
}
