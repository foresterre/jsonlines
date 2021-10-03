use camino::{Utf8Path, Utf8PathBuf};
use is_executable::IsExecutable;
use std::convert::TryFrom;
use std::process::{Command, Stdio};

// TODO: prettify :D
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();

    if args.len() >= 2 && &args[1] == "--list" {
        let output = collect_proxies().map(|proxies| {
            proxies
                .iter()
                .map(|proxy| format!("  * {} ({})", proxy.subject_name(), proxy.path()))
                .collect::<String>()
        })?;

        eprintln!("Available proxies:\n{}", output);
    } else if args.len() == 2 {
        let subject = &args[1];

        let proxies = collect_proxies()?;
        if let Some(proxy) = proxies.iter().find(|proxy| proxy.is_subject(subject)) {
            let mut child = proxy.start_process().spawn()?;
            let exit_status = child.wait()?;

            std::process::exit(exit_status.code().unwrap_or_default())
        } else {
            eprintln!("No such proxy found, try --list for a list of available options")
        }
    } else {
        eprintln!("No proxy command found, try --list for a list of available options");
    }

    Ok(())
}

fn collect_proxies() -> Result<Vec<Proxy>, Box<dyn std::error::Error>> {
    let proxies = std::env::var("PATH")?;

    let proxies = proxies
        .as_str()
        .split(";")
        .map(Utf8PathBuf::from)
        .collect::<Vec<Utf8PathBuf>>();

    Ok(proxies
        .iter()
        .flat_map(find_binaries)
        .collect::<Vec<Proxy>>())
}

const PREFIX: &str = "jsonlines-";

fn find_binaries<P: AsRef<Utf8Path>>(path: P) -> impl IntoIterator<Item = Proxy> {
    let mut buffer = Vec::with_capacity(100);

    if let Ok(dir) = std::fs::read_dir(path.as_ref()) {
        for proxy in dir
            .filter_map(|f| f.ok())
            .filter(|f| {
                f.file_type().map(|p| p.is_file()).unwrap_or_default() && f.path().is_executable()
            })
            .filter_map(|f| {
                Utf8PathBuf::try_from(f.path()).ok().and_then(|f| {
                    f.file_name()
                        .filter(|s| s.starts_with(PREFIX))
                        .map(|s| Proxy(f.to_owned(), s.to_string()))
                })
            })
        {
            buffer.push(proxy);
        }
    }

    buffer
}

/// Proxy of a path  
///
/// Upon proxy construction paths must start with the name of the referer,
/// and end in the name of the subject to which we can forward further process data.
#[derive(Debug)]
struct Proxy(Utf8PathBuf, String);

impl Proxy {
    #[inline]
    fn path(&self) -> &Utf8Path {
        &self.0
    }

    #[inline]
    fn file_name(&self) -> &str {
        &self.1
    }

    fn subject_name(&self) -> &str {
        let name = self.file_name().trim_start_matches(PREFIX);

        if std::env::consts::OS == "windows" {
            name.trim_end_matches(".exe")
        } else {
            name
        }
    }

    fn is_subject(&self, expected_name: &str) -> bool {
        self.subject_name() == expected_name
    }

    fn start_process(&self) -> Command {
        // Tbd: we do this twice, one in main() and once here, so we don't have to inject
        //  it again; should we?
        let mut args = std::env::args().collect::<Vec<String>>();

        // fake origin
        args[0] = self.1.to_owned();

        let mut cmd = Command::new(&self.0);
        cmd.args(std::env::args());
        cmd.stdin(Stdio::piped());
        cmd
    }
}
