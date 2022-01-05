# Simple Rust Git Repo Viewer

A git repo viewer and lfs server created for myself.

## Archetecture

## Deployment

### git-shell & git-lfs-authenticate

> TODO

### git-server

> TODO

## Configuration

Some environment variables are required as configurations:
```bash
# LFS
export AWS_ACCESS_KEY_ID=
export AWS_SECRET_ACCESS_KEY=
export AWS_BUCKET_NAME=
export AWS_ENDPOINT_REGION=
export AWS_ENDPOINT_PREFIX=

# logging
export RUST_LOG=info

# authorization between git-lfs-authenticate and git-server. maybe latter for other purpose
export SECRET=
```
