use std::{collections::HashMap, fmt::Display, str::FromStr};

use actix_web::{web, HttpResponse};
use futures::StreamExt;
use log::debug;
use serde::*;

use crate::middleware::token_extractor::Token;
use crate::AppContext;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LFSObjectURLAction {
    href: String,
    header: HashMap<String, String>,
    expires_in: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LFSObject {
    pub oid: String,
    pub size: usize,

    #[serde(skip_deserializing)]
    pub authenticated: bool,

    #[serde(skip_deserializing)]
    actions: HashMap<String, LFSObjectURLAction>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LFSReference {
    pub name: String,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize)]
pub enum LFSOperation {
    download,
    upload,
}

impl Display for LFSOperation {
    fn fmt(&self, f: &mut __private::Formatter<'_>) -> std::fmt::Result {
        match self {
            LFSOperation::download => write!(f, "download"),
            LFSOperation::upload => write!(f, "upload"),
        }?;
        Ok(())
    }
}

impl FromStr for LFSOperation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "download" => Ok(Self::download),
            "upload" => Ok(Self::upload),
            _ => Err(format!("unknown operation: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LFSBatchRequest {
    pub operation: LFSOperation,
    pub objects: Vec<LFSObject>,

    #[serde(default = "default_transfers")]
    pub transfers: Vec<String>,

    pub r#ref: LFSReference,
    #[serde(default = "default_hash_algo")]
    pub hash_algo: String,
}

fn default_hash_algo() -> String {
    "sha256".into()
}

fn default_transfers() -> Vec<String> {
    vec!["basic".into()]
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LFSBatchResponse {
    pub transfer: String,
    pub objects: Vec<LFSObject>,
}

#[actix_web::post("/{repo_path:.*\\.git}/info/lfs/locks/verify")]
pub async fn lfs_lock_verify(
    request: actix_web::HttpRequest,
    mut body: actix_web::web::Payload,
) -> Result<String, actix_web::error::Error> {
    println!("{:?}", request.headers());

    let mut bytes = actix_web::web::BytesMut::new();
    while let Some(item) = body.next().await {
        let item = item?;
        println!("Chunk: {:?}", &item);
        bytes.extend_from_slice(&item);
    }

    Err(actix_web::error::ErrorInternalServerError(""))
}

#[actix_web::post("/{repo_path:.*\\.git}/info/lfs/objects/batch")]
pub async fn lfs_objects_batch(
    web::Path(repo_path): web::Path<String>,
    body: web::Json<LFSBatchRequest>,
    appctx: actix_web::web::Data<AppContext>,
    token: Token,
) -> Result<HttpResponse, actix_web::error::Error> {
    let mut objects = Vec::new();

    assert_eq!(token.command, body.operation.to_string());

    for obj in body.objects.iter() {
        debug!("check object: {}", obj.oid.clone());
        let _appctx = appctx.clone();
        let _repo_path = repo_path.clone();
        let _obj = obj.clone();
        match web::block(move || {
            _appctx
                .bucket
                .head_object(format!("{}/lfs/objects/{}", _repo_path, _obj.oid.clone()))
        })
        .await
        {
            Ok(_) => continue,
            Err(_) => debug!("add object {} to list", obj.oid),
        }
        debug!("object need operation: {}", obj.oid.clone());
        let expires_in = 3600;
        let href = match body.operation {
            LFSOperation::download => appctx
                .bucket
                .presign_get(
                    format!("{}/lfs/objects/{}", repo_path, obj.oid.clone()),
                    expires_in.try_into().unwrap(),
                )
                .unwrap(),
            LFSOperation::upload => appctx
                .bucket
                .presign_put(
                    format!("{}/lfs/objects/{}", repo_path, obj.oid.clone()),
                    expires_in.try_into().unwrap(),
                    None,
                )
                .unwrap(),
        };

        let actions = match body.operation {
            LFSOperation::download => HashMap::from_iter(vec![(
                "download".into(),
                LFSObjectURLAction {
                    href,
                    expires_in,
                    header: HashMap::new(),
                },
            )]),
            LFSOperation::upload => HashMap::from_iter(vec![
                (
                    "upload".into(),
                    LFSObjectURLAction {
                        href,
                        expires_in,
                        header: HashMap::new(),
                    },
                ),
                // (
                //     "verify".into(),
                //     LFSObjectURLAction {
                //         href: format!("/{}/info/lfs/objects", repo_path),
                //         header: HashMap::from_iter(vec![("Authorization".into(), "Token eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJnaXQuamVmZnRoZWNvZGVyLnh5eiIsImlhdCI6IjIwMjItMDEtMDJUMTU6MDA6MDMuMTY1ODE3OTM5WiIsImNvb
                //         W1hbmQiOiJ1cGxvYWQifQ.Mt0mTyFQw-YOaLlJ_ZytfitpZnROus3UorE_NX30V-k".into())]),
                //         expires_in,
                //     },
                // ),
            ]),
        };
        debug!("object processed: {}", obj.oid.clone());

        objects.push(LFSObject {
            oid: obj.oid.clone(),
            size: obj.size,
            authenticated: true,
            actions: actions,
        });
    }

    Ok(HttpResponse::Ok().json(LFSBatchResponse {
        transfer: "basic".into(),
        objects: objects,
    }))
}
