use std::fs::File;
use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let (opts, opt_path) = match args.len() {
        3 => (Some(&args[1]), Some(&args[2])),
        2 => {
            if args[1].starts_with("-") {
                (Some(&args[1]), None)
            } else {
                (None, Some(&args[1]))
            }
        }
        1 => (None, None),
        _ => {
            eprintln!("Usage: ccwc <option> <file>");
            std::process::exit(1);
        }
    };

    match opt_path {
        Some(path) => {
            let mut file = File::open(path)
                .expect(&format!("ccwc: {}: open: No such file or directory", path));
            let result = process(&mut file, opts);
            println!(" {:>7} {}", result, path)
        }
        None => {
            let mut stdin_lock = std::io::stdin().lock();
            let result = process(&mut stdin_lock, opts);
            println!(" {:>7}", result);
        }
    }
}

fn process<R: Read>(r: &mut R, opts: Option<&String>) -> String {
    let counts = count_all(r);
    if let Some(opt) = opts {
        let result = match opt.as_str() {
            "-c" => counts.bytes,
            "-l" => counts.lines,
            "-w" => counts.words,
            "-m" => counts.chars,
            _ => {
                eprintln!("EITA! Usage: ccwc <option> <file>");
                std::process::exit(1);
            }
        };
        format!("{:>7}", result)
    } else {
        format!(
            "{:>7} {:>7} {:>7}",
            counts.lines, counts.words, counts.bytes
        )
    }
}

struct Counts {
    bytes: u64,
    lines: u64,
    words: u64,
    chars: u64,
}

fn count_all<R: Read>(r: &mut R) -> Counts {
    let mut buf = Vec::new();
    r.read_to_end(&mut buf).unwrap();
    let s = String::from_utf8_lossy(&buf);

    Counts {
        bytes: buf.len() as u64,
        lines: s.lines().count() as u64,
        words: s.split_whitespace().count() as u64,
        chars: s.chars().count() as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_file;
    use std::io::{Cursor, Write};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_file(content: &str) -> PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

        let mut path = std::env::temp_dir();
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        path.push(format!("ccwc_test_{}_{}.tmp", t, id));
        let mut f = File::create(&path).expect("create temp file");
        f.write_all(content.as_bytes()).expect("write temp file");
        path
    }

    #[test]
    fn test_count_bytes_file() {
        let path = make_temp_file("hello\n");
        let mut file = File::open(&path).expect("open temp file");
        let n = count_all(&mut file).bytes;
        assert_eq!(n, 6);
        remove_file(path).unwrap();
    }

    #[test]
    fn test_count_lines_file() {
        let path = make_temp_file("a\nb\nc\n");
        let mut file = File::open(&path).expect("open temp file");
        let n = count_all(&mut file).lines;
        assert_eq!(n, 3);
        remove_file(path).unwrap();
    }

    #[test]
    fn test_count_words_file() {
        let path = make_temp_file("one two\tthree\nfour");
        let mut file = File::open(&path).expect("open temp file");
        let n = count_all(&mut file).words;
        assert_eq!(n, 4);
        remove_file(path).unwrap();
    }

    #[test]
    fn test_count_characters_file() {
        let path = make_temp_file("hélló");
        let mut file = File::open(&path).expect("open temp file");
        let n = count_all(&mut file).chars;
        assert_eq!(n, 5);
        remove_file(path).unwrap();
    }

    #[test]
    fn test_count_bytes_stdin() {
        let mut stdin = Cursor::new(b"hello\n");
        let n = count_all(&mut stdin).bytes;
        assert_eq!(n, 6);
    }

    #[test]
    fn test_count_lines_stdin() {
        let mut stdin = Cursor::new(b"a\nb\nc\n");
        let n = count_all(&mut stdin).lines;
        assert_eq!(n, 3);
    }

    #[test]
    fn test_count_words_stdin() {
        let mut stdin = Cursor::new(b"one two\tthree\nfour");
        let n = count_all(&mut stdin).words;
        assert_eq!(n, 4);
    }

    #[test]
    fn test_count_characters_stdin() {
        let mut stdin = Cursor::new("hélló".as_bytes());
        let n = count_all(&mut stdin).chars;
        assert_eq!(n, 5);
    }
}
