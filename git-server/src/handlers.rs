use std::path::Path;

use actix_web::{web, HttpResponse};
use askama_actix::TemplateIntoResponse;
use git2::{BranchType, ObjectType, Oid};

use crate::templates::*;

fn extract_repo_info(repo: &git2::Repository) -> (Vec<String>, Vec<String>) {
    let mut branches = Vec::new();
    let mut tags = Vec::new();
    if let Ok(references) = repo.references() {
        for reference in references {
            if let Ok(r) = reference {
                if let Ok(name) = String::from_utf8(r.shorthand_bytes().to_vec()) {
                    if r.is_branch() {
                        branches.push(name.clone());
                        if name == "master" || name == "main" {
                            if let Ok(commit) = r.peel_to_commit() {
                                if let Ok(tree) = commit.tree() {
                                    if let Some(entry) = tree.get_name("README.md") {
                                        if let Ok(obj) = entry.to_object(&repo) {
                                            println!("{:?}", obj)
                                        }
                                    }
                                }
                            }
                        }
                    } else if r.is_tag() {
                        tags.push(name);
                    }
                }
            }
        }
    }
    (branches, tags)
}

fn extract_oid_from_name(repo: &git2::Repository, name: String) -> Result<Oid, git2::Error> {
    println!("{:?}", repo.path());
    match repo.branches(None) {
        Ok(branches) => {
            for branch in branches {
                match branch {
                    Ok((branch, kind)) => {
                        println!("{:?}, {:?}", branch.name(), kind);
                    }
                    Err(_) => todo!(),
                }
            }
        }
        Err(_) => todo!(),
    };
    match repo.find_branch(&name, BranchType::Local) {
        Ok(branch) => match branch.get().peel_to_tree() {
            Ok(tree) => {
                return Ok(tree.id());
            }
            Err(err) => return Err(err),
        },
        Err(err) => {
            println!("failed to get branch: {}", err);
        }
    }
    repo.tag_foreach(|oid, name| {
        println!("{:?}", String::from_utf8(name.to_vec()));
        return true;
    })?;
    println!("extracting oid from name: {}", name);
    match Oid::from_str(name.as_str()) {
        Ok(oid) => Ok(oid),
        Err(err) => Err(err),
    }
}

async fn git_repo_page(
    repo_path: String,
    ref_name: String,
    object_path: String,
) -> Result<impl actix_web::Responder, actix_web::Error> {
    let _repo_path = repo_path.clone();
    let _ref_name = ref_name.clone();
    let _object_path = object_path.clone();
    let (branches, tags, _spec_oid, spec_kind, path_oid, path_type) = web::block(move || {
        let repo = match git2::Repository::open_bare(_repo_path.clone()) {
            Ok(repo) => repo,
            Err(err) => {
                return Err(format!("failed to open repo: {}", err));
            }
        };

        let (branches, tags) = extract_repo_info(&repo);

        let spec_oid = match extract_oid_from_name(&repo, _ref_name.clone()) {
            Ok(oid) => oid,
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
                if let Ok(tree) = spec_object.peel_to_tree() {
                    if let Ok(entry) = tree.get_path(Path::new(_object_path.as_str())) {
                        path_oid = Some(entry.id());
                        path_type = entry.kind();
                    }
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

    if spec_kind == ObjectType::Commit
        || spec_kind == ObjectType::Tag
        || spec_kind == ObjectType::Tree
    {
        println!("path_type: {:?}", path_type);

        if let Some(kind) = path_type {
            if kind == ObjectType::Tree {
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
                            entries.push(String::from(name));
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
                        repo_path,
                        branches,
                        tags,
                    },
                    object_path,
                    entries,
                    ref_name,
                    readme,
                };

                return match page.into_response() {
                    Ok(response) => Ok(response),
                    Err(err) => Err(actix_web::error::ErrorInternalServerError(err)),
                };
            } else if kind == ObjectType::Blob {
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

                    let oid = match extract_oid_from_name(&repo, _ref_name.clone()) {
                        Ok(oid) => oid,
                        Err(err) => {
                            return Err(format!(
                                "failed to extract oid from name '{}': {:?}",
                                _ref_name.clone(),
                                err
                            ))
                        }
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
                            entries.push(String::from(name));
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
                        repo_path,
                        branches,
                        tags,
                    },
                    object_path,
                    entries,
                    ref_name,
                    readme,
                };

                return match page.into_response() {
                    Ok(response) => Ok(response),
                    Err(err) => Err(actix_web::error::ErrorInternalServerError(err)),
                };
            } else {
                todo!();
            }
        } else {
            todo!();
        }
    } else if spec_kind == ObjectType::Blob {
        let page = GitBlobPage {
            _parent: GitBaseTemplate {
                _parent: BaseTemplate::new().with_title(repo_path.clone()),
                repo_path,
                branches,
                tags,
            },
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

#[actix_web::get("/{repo_path:.*\\.git}/tree/{ref_name}/{object_path:.*}")]
async fn git_repo_detail(
    web::Path((repo_path, ref_name, object_path)): web::Path<(String, String, String)>,
) -> Result<impl actix_web::Responder, actix_web::Error> {
    git_repo_page(repo_path, ref_name, object_path).await
}

#[actix_web::get("/{path:.*\\.git}")]
async fn git_repo(
    web::Path(repo_path): web::Path<String>,
) -> Result<impl actix_web::Responder, actix_web::Error> {
    git_repo_page(repo_path, String::from("master"), String::from("/")).await
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

#[actix_web::post("/{repo_path:.*\\.git}/{ref_name}/info/lfs")]
async fn git_lfs_batch() -> impl actix_web::Responder {
    ""
}
