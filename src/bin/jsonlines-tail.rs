use std::io::BufRead;

fn main() -> Result<(), Error> {
    let stdin = std::io::stdin();
    let mut stdin_lock = stdin.lock();

    let mut last = NextLine::Empty;

    while let Ok(line) = read_json_line(&mut stdin_lock) {
        if let NextLine::Empty = line {
            break;
        }

        last = line;
    }

    match last {
        NextLine::Empty => Err(Error::Empty),
        NextLine::Value(v) => {
            println!("{}", v);
            Ok(())
        }
    }
}

fn read_json_line<R: BufRead>(src: &mut R) -> Result<NextLine, Error> {
    let mut line = String::new();

    let _result = src.read_line(&mut line).map_err(Error::Io)?;
    let trimmed = line.trim();

    if trimmed.len() == 0 {
        return Ok(NextLine::Empty);
    }

    json::parse(trimmed)
        .map(|line| NextLine::Value(line.dump()))
        .map_err(|err| Error::InvalidJson(err))
}

#[derive(Debug)]
enum NextLine {
    Value(String),
    Empty,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Stdin was empty")]
    Empty,

    #[error("Line was invalid: {0}")]
    InvalidJson(json::Error),

    #[error("Unable to read: {0}")]
    Io(std::io::Error),
}
