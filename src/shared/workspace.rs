#[derive(Debug)]
pub struct Workspace {
    pub name: Option<String>,
    pub add: bool,
    pub delete: bool,
    pub rename: Option<String>,
    pub include: Option<String>,
    pub remove: Option<String>,
    pub list: bool,
}
