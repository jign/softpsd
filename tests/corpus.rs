#![cfg(feature = "corpus")]

use std::{fs, io::Write, panic::catch_unwind, path::Path};

fn files(path: &Path, found: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let path = entry.path();
        if kind.is_dir() && entry.file_name() != ".git" {
            files(&path, found)?;
        } else if kind.is_file()
            && path.extension().is_some_and(|ext| {
                ext.eq_ignore_ascii_case("psd") || ext.eq_ignore_ascii_case("psb")
            })
        {
            found.push(path);
        }
    }
    Ok(())
}

#[test]
fn corpus_never_panics() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut found = Vec::new();
    files(&root.join("corpus"), &mut found).expect("run tools/fetch-corpus.ps1 first");
    found.sort();
    fs::create_dir_all(root.join("target")).unwrap();
    let mut report = fs::File::create(root.join("target/corpus.txt")).unwrap();
    let mut panics = 0;
    for path in &found {
        let status = if fs::metadata(path).unwrap().len() > 64 * 1024 * 1024 {
            String::from("skipped: size")
        } else {
            let bytes = fs::read(path).unwrap();
            match catch_unwind(|| softpsd::read(&bytes)) {
                Ok(Ok(_)) => String::from("ok"),
                Ok(Err(error)) => error.to_string().replace(['\r', '\n', '\t'], " "),
                Err(_) => {
                    panics += 1;
                    String::from("panic")
                }
            }
        };
        let path = path
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        writeln!(report, "{status}\t{path}").unwrap();
    }
    assert!(!found.is_empty(), "corpus contains no PSD/PSB files");
    assert!(
        panics == 0,
        "{panics} corpus files panicked; see target/corpus.txt"
    );
}
