use std::cmp::Ordering;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "webp", "heic", "heif", "bmp", "tiff", "mp4", "mov", "m4v", "webm",
];

pub fn is_media(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|n| n.as_encoded_bytes().starts_with(b"."))
}

/// Finder-like order: case-insensitive, digit runs compared by value (IMG_2 < IMG_10).
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (mut x, mut y) = (a, b);
    while let (Some(cx), Some(cy)) = (x.chars().next(), y.chars().next()) {
        let ord = if cx.is_ascii_digit() && cy.is_ascii_digit() {
            let (nx, rx) = split_digits(x);
            let (ny, ry) = split_digits(y);
            (x, y) = (rx, ry);
            // compare as strings, not u64: digit runs can be arbitrarily long
            let (tx, ty) = (nx.trim_start_matches('0'), ny.trim_start_matches('0'));
            tx.len()
                .cmp(&ty.len())
                .then(tx.cmp(ty))
                .then(nx.len().cmp(&ny.len()))
        } else {
            (x, y) = (&x[cx.len_utf8()..], &y[cy.len_utf8()..]);
            cx.to_lowercase().cmp(cy.to_lowercase())
        };
        if ord != Ordering::Equal {
            return ord;
        }
    }
    x.len().cmp(&y.len()).then_with(|| a.cmp(b))
}

fn split_digits(s: &str) -> (&str, &str) {
    s.split_at(s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len()))
}

/// Media files of `dir`, naturally sorted, plus the index of `current`.
/// `current` is always listed, even if hidden or not a media type.
/// `dir` and `current` must already be canonical.
pub fn list_media(dir: &Path, current: &Path) -> io::Result<(Vec<PathBuf>, usize)> {
    let Ok(entries) = fs::read_dir(dir) else {
        // unreadable folder (TCC-denied ~/Downloads, 711 permissions): still show the opened file
        return Ok((vec![current.to_path_buf()], 0));
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && (p == current || (is_media(p) && !is_hidden(p))))
        .collect();
    files.sort_by(|a, b| {
        natural_cmp(
            &a.file_name().unwrap().to_string_lossy(),
            &b.file_name().unwrap().to_string_lossy(),
        )
    });
    let index = files
        .iter()
        .position(|p| p == current)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file is not in its folder"))?;
    Ok((files, index))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn natural_order() {
        let mut v = vec![
            "IMG_10.jpg",
            "img_2.JPG",
            "IMG_1.jpg",
            "b.png",
            "A.png",
            "IMG_002.jpg",
        ];
        v.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(
            v,
            [
                "A.png",
                "b.png",
                "IMG_1.jpg",
                "img_2.JPG",
                "IMG_002.jpg",
                "IMG_10.jpg"
            ]
        );
    }

    #[test]
    fn natural_order_long_digit_run_no_overflow() {
        assert_eq!(
            natural_cmp(
                "x99999999999999999999999.jpg",
                "x100000000000000000000000.jpg"
            ),
            Ordering::Less
        );
    }

    #[test]
    fn media_filter_case_insensitive() {
        assert!(is_media(Path::new("/a/B.HEIC")));
        assert!(is_media(Path::new("/a/clip.WebM")));
        assert!(!is_media(Path::new("/a/notes.txt")));
        assert!(!is_media(Path::new("/a/noext")));
    }

    fn fixture(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("riffle-test-{}-{name}", std::process::id()))
            .join("Фото [2024] #1");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("sub.jpg")).unwrap();
        for f in [
            "IMG_10.jpg",
            "IMG_2.JPG",
            ".hidden.jpg",
            "._IMG_2.JPG",
            "notes.txt",
        ] {
            fs::write(dir.join(f), b"x").unwrap();
        }
        dir.canonicalize().unwrap()
    }

    fn names(files: &[PathBuf]) -> Vec<&str> {
        files
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect()
    }

    #[test]
    fn lists_folder_skips_hidden_dirs_and_others() {
        let dir = fixture("list");
        let (files, idx) = list_media(&dir, &dir.join("IMG_10.jpg")).unwrap();
        assert_eq!(names(&files), ["IMG_2.JPG", "IMG_10.jpg"]);
        assert_eq!(idx, 1);
        fs::remove_dir_all(dir.parent().unwrap()).unwrap();
    }

    #[test]
    fn unreadable_folder_still_lists_the_opened_file() {
        use std::os::unix::fs::PermissionsExt;
        let dir = fixture("noread");
        // like a TCC-denied ~/Downloads or a 711 folder: files openable by name, folder not listable
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o311)).unwrap();
        let result = list_media(&dir, &dir.join("IMG_10.jpg"));
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
        let (files, idx) = result.unwrap();
        assert_eq!(names(&files), ["IMG_10.jpg"]);
        assert_eq!(idx, 0);
        fs::remove_dir_all(dir.parent().unwrap()).unwrap();
    }

    #[test]
    fn explicitly_opened_file_is_always_in_list() {
        let dir = fixture("hidden");
        let (files, idx) = list_media(&dir, &dir.join(".hidden.jpg")).unwrap();
        assert_eq!(names(&files), [".hidden.jpg", "IMG_2.JPG", "IMG_10.jpg"]);
        assert_eq!(idx, 0);
        fs::remove_dir_all(dir.parent().unwrap()).unwrap();
    }
}
