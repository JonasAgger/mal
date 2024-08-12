use std::io::Write;

pub struct Console {
    // stdin: StdinLock<'static>,
    // stdout: StdoutLock<'static>,
}

impl Console {
    pub fn read_user_input() -> Option<String> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        stdout.write_all(b"user> ").ok()?;
        stdout.flush().ok()?;

        let mut buffer = String::new();

        if stdin.read_line(&mut buffer).ok()? == 0 {
            return None;
        }

        if buffer.ends_with('\n') {
            buffer.pop();

            if buffer.ends_with('\r') {
                buffer.pop();
            }
        }

        if buffer.ends_with(0x05 as char) {
            return None;
        }

        if buffer.len() > 0 {
            Some(buffer)
        } else {
            None
        }
    }
}
