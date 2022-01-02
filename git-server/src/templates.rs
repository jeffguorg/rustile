use askama::Template;

#[derive(Template)]
#[template(path = "_base.html")]
pub struct BaseTemplate {
    pub title: Option<String>,
}

impl BaseTemplate {
    pub fn new() -> Self {
        Self { title: None }
    }

    pub fn with_title(self, title: String) -> Self {
        Self { title: Some(title) }
    }
}

#[derive(Template)]
#[template(path = "_git_base.html")]
pub struct GitBaseTemplate {
    pub _parent: BaseTemplate,

    pub repo_path: String,
    pub branches: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Template)]
#[template(path = "git_tree_page.html")]
pub struct GitTreePage {
    pub _parent: GitBaseTemplate,
    pub ref_name: String,
    pub object_path: String,
    pub entries: Vec<String>,
    pub readme: Option<String>,
}

#[derive(Template)]
#[template(path = "git_blob_page.html")]
pub struct GitBlobPage {
    pub _parent: GitBaseTemplate,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexPage {
    pub paths: Vec<String>,
}
