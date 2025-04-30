use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    #[arg(short = 'L', long, default_value = "http://:8080")]
    listen: String,

    #[arg(short = 'F', long, default_value = "http://192.168.1.1:8080")]
    forward: String,
}

impl Cli {
    pub(crate) fn get_listen(&self) -> &str {
        &self.listen
    }

    pub(crate) fn get_forward(&self) -> &str {
        &self.forward
    }
}
