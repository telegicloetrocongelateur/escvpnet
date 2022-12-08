pub enum Command<'a> {
    Set {
        command: &'a str,
        parameter: &'a str,
    },
    Get {
        command: &'a str,
    },
    Null,
}

pub struct Response {
    command: String,
    parameter: String,
}

impl Response {
    pub fn new(command: String, parameter: String) -> Self {
        Self { command, parameter }
    }
    pub fn command(self) -> String {
        self.command
    }

    pub fn paramter(self) -> String {
        self.parameter
    }
}
