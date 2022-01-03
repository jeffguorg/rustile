use askama::Template;
use git2::ObjectType;

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
    pub ref_name: String,
    pub spec_kind: ObjectType,
    pub object_type: String,
    pub object_path: Option<String>,
    pub branches: Vec<String>,
    pub tags: Vec<String>,

    pub breadcrumb: Vec<(String, String)>,
}

pub struct Entry {
    pub name: String,
    pub kind: ObjectType,
}

#[derive(Template)]
#[template(path = "git_tree_page.html")]
pub struct GitTreePage {
    pub _parent: GitBaseTemplate,
    pub entries: Vec<Entry>,
    pub readme: Option<String>,
}

#[derive(Template)]
#[template(path = "git_blob_page.html")]
pub struct GitBlobPage {
    pub _parent: GitBaseTemplate,

    pub text_content: Option<String>,
    pub size: usize,
}

#[derive(Template)]
#[template(path = "file_browser.html")]
pub struct FileBrowserPage {
    pub path: String,
    pub entries: Vec<String>,
}
