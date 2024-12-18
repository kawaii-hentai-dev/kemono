#[derive(Default, Debug, Clone, PartialEq)]
pub struct Attachment<'a> {
    pub file_server: &'a str,
    pub file_name: &'a str,
    pub file_path: &'a str,
}
