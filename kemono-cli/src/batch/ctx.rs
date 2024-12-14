use std::path::PathBuf;

use derive_builder::Builder;

pub trait Context<'a> {
    fn web_name(&self) -> &'a str;
    fn user_id(&self) -> &'a str;
    fn output_dir(&self) -> &'a PathBuf;
    fn max_concurrency(&self) -> usize;
}

#[derive(Clone, Builder, PartialEq, Eq, Default)]
pub struct Args {
    web_name: String,
    user_id: String,
    output_dir: PathBuf,
    max_concurrency: usize,
}

impl Args {
    pub fn builder() -> ArgsBuilder {
        ArgsBuilder::default()
    }
}

impl<'a> Context<'a> for &'a Args {
    fn web_name(&self) -> &'a str {
        &self.web_name
    }

    fn user_id(&self) -> &'a str {
        &self.user_id
    }

    fn max_concurrency(&self) -> usize {
        self.max_concurrency
    }

    fn output_dir(&self) -> &'a PathBuf {
        &self.output_dir
    }
}
