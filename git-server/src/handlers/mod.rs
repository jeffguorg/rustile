use std::path::Path;

use actix_web::{web, HttpResponse};
use askama_actix::TemplateIntoResponse;
use git2::{BranchType, ObjectType, Oid};

pub use lfs::*;

mod lfs;

use crate::templates::*;

lazy_static::lazy_static! {
    static ref TAG_CAPTURE: regex::Regex = regex::Regex::new("refs/tags/(?P<tag_name>.*)").unwrap();
}

fn extract_tag_shortname(s: String) -> Option<String> {
    match TAG_CAPTURE.captures(&s) {
        Some(captures) => captures.name("tag_name").map(|m| String::from(m.as_str())),
        None => None,
    }
}

fn extract_repo_info(repo: &git2::Repository) -> (Vec<String>, Vec<String>) {
    let mut branches = Vec::new();
    let mut tags = Vec::new();
    if let Ok(repo_branches) = repo.branches(Some(BranchType::Local)) {
        for branch in repo_branches {
            if let Ok((branch, _)) = branch {
                if let Ok(name) = branch.name() {
                    if let Some(name) = name {
                        branches.push(String::from(name));
                    }
                }
            }
        }
    }
    repo.tag_foreach(|_, tag_name| {
        if let Ok(tag) = String::from_utf8(tag_name.to_vec()) {
            if let Some(shortname) = extract_tag_shortname(tag) {
                tags.push(shortname);
            }
        }
        true
    })
    .unwrap();
    (branches, tags)
}

fn extract_oid_from_name(
    repo: &git2::Repository,
    name: String,
) -> Result<Option<Oid>, git2::Error> {
    match repo.find_branch(&name, BranchType::Local) {
        Ok(branch) => match branch.get().peel_to_tree() {
            Ok(tree) => {
                return Ok(Some(tree.id()));
            }
            Err(err) => return Err(err),
        },
        Err(_err) => {
            // println!("failed to get branch: {}", _err);
        }
    }
    let mut maybe_oid = None;
    repo.tag_foreach(|oid, unparsed| {
        if let Ok(parsed) = String::from_utf8(unparsed.to_vec()) {
            if let Some(shorthand) = extract_tag_shortname(parsed) {
                if name == shorthand {
                    maybe_oid = Some(oid);
                }
            }
        }
        return true;
    })?;
    if maybe_oid != None {
        return Ok(maybe_oid);
    }
    match Oid::from_str(name.as_str()) {
        Ok(oid) => Ok(Some(oid)),
        Err(err) => Err(err),
    }
}

async fn git_repo_page(
    repo_path: String,
    object_type: String,
    ref_name: String,
    object_path: Option<String>,
) -> Result<impl actix_web::Responder, actix_web::Error> {
    let _repo_path = repo_path.clone();
    let _ref_name = ref_name.clone();
    let _object_path = object_path.clone();
    let (branches, tags, _spec_oid, spec_kind, path_oid, path_type): (
        Vec<String>,
        Vec<String>,
        Oid,
        ObjectType,
        Option<Oid>,
        Option<ObjectType>,
    ) = web::block(move || {
        let repo = match git2::Repository::open_bare(_repo_path.clone()) {
            Ok(repo) => repo,
            Err(err) => {
                return Err(format!("failed to open repo: {}", err));
            }
        };

        let (branches, tags) = extract_repo_info(&repo);

        let spec_oid = match extract_oid_from_name(&repo, _ref_name.clone()) {
            Ok(oid) => match oid {
                Some(oid) => oid,
                None => todo!(),
            },
            Err(err) => {
                return Err(format!(
                    "failed to extract oid from name '{}': {:?}.",
                    _ref_name.clone(),
                    err
                ))
            }
        };

        let spec_object = match repo.find_object(spec_oid, None) {
            Ok(obj) => obj,
            Err(err) => return Err(format!("failed to find object: {}", err)),
        };

        let mut path_oid = None;
        let mut path_type = None;
        if let Some(kind) = spec_object.kind() {
            if kind != ObjectType::Blob {
                match spec_object.peel_to_tree() {
                    Ok(tree) => {
                        if let Some(object_path) = _object_path {
                            match tree.get_path(Path::new(object_path.as_str())) {
                                Ok(entry) => {
                                    path_oid = Some(entry.id());
                                    path_type = entry.kind();
                                }
                                Err(err) => eprintln!("failed to get entry on path: {}", err),
                            }
                        } else {
                            path_oid = Some(tree.id());
                            path_type = Some(ObjectType::Tree);
                        }
                    }
                    Err(err) => eprintln!("falied to peel to tree: {}", err),
                }
            }
        }

        return Ok((
            branches,
            tags,
            spec_oid,
            spec_object.kind().unwrap(),
            path_oid,
            path_type,
        ));
    })
    .await?;

    if path_type.unwrap() == ObjectType::Tree && object_type == "tree" {
        let _repo_path = repo_path.clone();
        let _object_path = object_path.clone();
        let _ref_name = ref_name.clone();
        let (entries, readme) = web::block(move || {
            let repo = match git2::Repository::open_bare(_repo_path.clone()) {
                Ok(repo) => repo,
                Err(err) => {
                    return Err(format!("no such repo: {:?}", err));
                }
            };

            let oid = match path_oid {
                Some(oid) => oid,
                None => return Err(format!("no such id: {:?}", _ref_name)),
            };

            let obj = match repo.find_object(oid, None) {
                Ok(obj) => obj,
                Err(err) => return Err(format!("failed to find object: {}", err)),
            };
            let tree = match obj.peel_to_tree() {
                Ok(tree) => tree,
                Err(err) => return Err(format!("failed to find tree: {}", err)),
            };

            let mut entries = Vec::new();
            let mut readme_blob = None;
            for entry in tree.iter() {
                if let Some(name) = entry.name() {
                    entries.push((String::from(name), entry.kind().unwrap()));
                    if name == "README.md" || name == "README" {
                        if let Some(kind) = entry.kind() {
                            if kind == ObjectType::Blob {
                                if let Ok(blob_obj) = entry.to_object(&repo) {
                                    readme_blob = match blob_obj.as_blob() {
                                        Some(blob) => Some(blob.clone()),
                                        None => None,
                                    }
                                }
                            }
                        }
                    }
                }
            }
            let readme = match readme_blob {
                None => None,
                Some(blob) => {
                    if blob.is_binary() {
                        None
                    } else {
                        if let Ok(s) = String::from_utf8(blob.content().to_vec()) {
                            Some(s)
                        } else {
                            None
                        }
                    }
                }
            };

            Ok((entries, readme))
        })
        .await?;

        let page = GitTreePage {
            _parent: GitBaseTemplate {
                _parent: BaseTemplate::new().with_title(repo_path.clone()),
                repo_path: repo_path.clone(),
                branches,
                ref_name: ref_name.clone(),
                spec_kind,
                object_type: object_type.clone(),
                object_path: object_path.clone(),
                tags,

                breadcrumb: match object_path {
                    Some(s) => {
                        let mut result = vec![(repo_path.clone(), String::new())];

                        let mut pieces = Vec::new();
                        for piece in s.split("/") {
                            println!("{}", piece);
                            pieces.push(String::from(piece));
                            result.push((String::from(piece), pieces.join("/")))
                        }

                        result
                    }
                    None => vec![(repo_path.clone(), String::new())],
                },
            },
            entries: entries
                .iter()
                .map(|(name, kind)| Entry {
                    name: name.clone(),
                    kind: kind.clone(),
                })
                .collect(),
            readme,
        };

        return match page.into_response() {
            Ok(response) => Ok(response),
            Err(err) => Err(actix_web::error::ErrorInternalServerError(err)),
        };
    } else if path_type.unwrap() == ObjectType::Blob && object_type == "blob" {
        let _repo_path = repo_path.clone();
        let _ref_name = ref_name.clone();
        let (text_content, size) = web::block(move || {
            let repo = match git2::Repository::open_bare(_repo_path.clone()) {
                Ok(repo) => repo,
                Err(err) => {
                    return Err(format!("no such repo: {:?}", err));
                }
            };

            let oid = match path_oid {
                Some(oid) => oid,
                None => return Err(format!("no such id: {:?}", _ref_name)),
            };

            let obj = match repo.find_object(oid, None) {
                Ok(obj) => obj,
                Err(err) => return Err(format!("failed to find object: {}", err)),
            };
            let blob = match obj.peel_to_blob() {
                Ok(blob) => blob,
                Err(err) => return Err(format!("failed to find tree: {}", err)),
            };

            let size = blob.size();
            let text_content = if blob.is_binary() {
                None
            } else {
                Some(match String::from_utf8(blob.content().to_vec()) {
                    Ok(s) => s,
                    Err(err) => return Err(format!("failed to find tree: {}", err)),
                })
            };

            Ok((text_content, size))
        })
        .await?;
        let page = GitBlobPage {
            _parent: GitBaseTemplate {
                _parent: BaseTemplate::new().with_title(repo_path.clone()),
                repo_path: repo_path.clone(),
                branches,
                tags,

                ref_name: ref_name.clone(),
                spec_kind,

                object_type,
                object_path: object_path.clone(),

                breadcrumb: match object_path {
                    Some(s) => {
                        let mut result = vec![(repo_path.clone(), String::new())];

                        let mut pieces = Vec::new();
                        for piece in s.split("/") {
                            println!("{}", piece);
                            pieces.push(String::from(piece));
                            result.push((String::from(piece), pieces.join("/")))
                        }

                        result
                    }
                    None => vec![(repo_path.clone(), String::new())],
                },
            },

            text_content,
            size,
        };

        match page.into_response() {
            Ok(response) => Ok(response),
            Err(err) => Err(actix_web::error::ErrorInternalServerError(err)),
        }
    } else {
        Err(actix_web::error::ErrorBadRequest(format!(
            "invalid reference type: {}",
            spec_kind
        )))
    }
}

#[actix_web::get("/{repo_path:.*\\.git}/{object_type:(tree|blob)}/{ref_name}/{object_path:.*}")]
async fn git_repo_detail(
    web::Path((repo_path, object_type, ref_name, object_path)): web::Path<(
        String,
        String,
        String,
        String,
    )>,
) -> Result<impl actix_web::Responder, actix_web::Error> {
    if object_path != "" {
        git_repo_page(repo_path, object_type, ref_name, Some(object_path)).await
    } else {
        git_repo_page(repo_path, object_type, ref_name, None).await
    }
}

#[actix_web::get("/{path:.*\\.git}/")]
async fn git_repo(
    web::Path(repo_path): web::Path<String>,
) -> Result<impl actix_web::Responder, actix_web::Error> {
    git_repo_page(
        repo_path,
        String::from("tree"),
        String::from("master"),
        None,
    )
    .await
}

#[actix_web::get("/{path:.*}")]
async fn index(web::Path(path): web::Path<String>) -> actix_web::Result<HttpResponse> {
    let mut path = path;
    if path == "" {
        path = String::from(".");
    }
    let entries = web::block(move || match std::fs::read_dir(path) {
        Ok(read_dir) => {
            let mut v = Vec::new();
            for entry in read_dir {
                if let Ok(entry) = entry {
                    if let Some(s) = entry.file_name().to_str() {
                        if String::from(s).starts_with(".") {
                            continue;
                        }
                    }
                    let path = entry.path();
                    if let Ok(s) = path.into_os_string().into_string() {
                        v.push(s);
                    }
                }
            }
            Ok(v)
        }
        Err(err) => Err(err),
    })
    .await?;

    Ok(IndexPage { paths: entries }.into_response()?)
}
