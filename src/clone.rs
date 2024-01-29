use anyhow::{bail, Result};
use bytes::Buf;
use bytes::Bytes;
use reqwest::get;
use std::path::Path;
use tokio::sync::broadcast;

use crate::init;
use crate::utils;
use crate::utils::create_directory;
use crate::utils::decompress;

pub async fn clone(uri: &str, target_dir: &str) -> Result<()> {
    // create directory
    let target_directory = Path::new(".").join(target_dir);

    create_directory(&target_directory)?;
    init::init(target_directory);

    // let refs = discover_references(uri).await?;
    // let commit = get_commit(&refs[0].commit_hash, uri).await?;
    // let commit_ref = CommitRef::new(commit)?;

    // dbg!(commit_ref);

    Ok(())
}

async fn discover_references(repo_uri: &str) -> Result<Vec<GitRef>> {
    let uri = format!("{repo_uri}/info/refs?service=git-upload-pack");
    let result = get(&uri).await?;
    let status = result.status();
    let response = result.bytes().await?;
    let header = response.slice(0..5);

    if !status.is_success() {
        bail!("failed request to discover references");
    }

    if !validate_header(&header) {
        bail!("Invalid header");
    }

    let references = process_ref_discovery_response(&response.slice(34..))?;

    Ok(references)
}

fn validate_header(header: &Bytes) -> bool {
    if header.len() != 5 {
        eprintln!("got header length {}, needed 5", header.len());
        return false;
    };

    if header[4] != b'#' {
        eprintln!("header doesn't end with '#'");
        return false;
    };

    true
}

fn process_ref_discovery_response(response: &Bytes) -> Result<Vec<GitRef>> {
    let responses = response.split(|b| *b == b'\n');
    let mut branch_refs = vec![];

    for line in responses.skip(1) {
        if line == b"0000" {
            break;
        }

        let mode = String::from_utf8(line[0..4].to_vec())?;
        let hash = String::from_utf8(line[4..44].to_vec())?;
        let branches = &line[45..].split(|branch| *branch == b'/');

        let branch_title = branches.clone().skip(1).next().expect("doesn't have title");
        if branch_title != b"heads" {
            break;
        }

        let branch = String::from_utf8(
            branches
                .clone()
                .last()
                .expect("couldn't find the branch name")
                .to_vec(),
        )?;
        let branch_ref = GitRef::new(&mode, &hash, &branch);

        branch_refs.push(branch_ref);
    }

    Ok(branch_refs)
}

#[derive(PartialEq, Debug)]
struct GitRef {
    mode: String,
    commit_hash: String,
    branch_name: String,
}

impl GitRef {
    pub fn new(mode: &str, commit_hash: &str, branch_name: &str) -> Self {
        Self {
            mode: mode.to_owned(),
            commit_hash: commit_hash.to_owned(),
            branch_name: branch_name.to_owned(),
        }
    }
}

#[derive(Default, Debug)]
enum ReaderState {
    #[default]
    Mode,
    Hash,
    BranchName,
    Features,
}

async fn get_commit(commit_hash: &str, repo_uri: &str) -> Result<Bytes> {
    let uri = format!("{repo_uri}/git-upload-pack");
    let client = reqwest::Client::new();
    let body = format!("0032want {commit_hash}\n00000009done\n");
    let response = client
        .post(uri)
        .header("Content-Type", "application/x-git-upload-pack-request")
        .body(body)
        .send()
        .await?;

    if !response.status().is_success() {
        bail!("Error response when getting commit");
    }

    let body = response.bytes().await?;

    Ok(body)
}

#[derive(Debug)]
struct CommitRef {
    head: String,
    signature: String,
    version: u32,
    object_count: u32,
}

impl CommitRef {
    pub fn new(commit: Bytes) -> Result<Self> {
        // let head = std::str::from_utf8(&commit[0..8])?.to_owned();
        // let signature = std::str::from_utf8(&commit[8..12])?.to_owned();
        // let version = u32::from_be_bytes(commit[12..16].try_into()?);
        // let object_count = u32::from_be_bytes(commit[16..20].try_into()?);
        // let object_type = &commit[20..22];
        // let object_type_header = format!("{object_type:b}");

        // dbg!(&object_type_header[0..1]);
        // dbg!(&object_type_header[1..4]); // type
        // dbg!(&object_type_header[4..]);

        // Ok(Self {
        //     head,
        //     signature,
        //     version,
        //     object_count,
        // })
        todo!()
    }
}

// enum ObjectType {
//     Commit,
//     Unknown,
// }

// impl ObjectType {
//     pub fn new(bits: &str) -> Self {}
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_validate_header() -> Result<()> {
        let header = "001e#";
        let expected_result = true;
        let result = validate_header(&header.into());

        assert_eq!(result, expected_result);
        Ok(())
    }

    #[test]
    fn an_empty_header_should_be_invalid() -> Result<()> {
        let header = "";
        let expected_result = false;
        let result = validate_header(&header.into());

        assert_eq!(result, expected_result);
        Ok(())
    }

    #[test]
    fn should_extract_one_ref() -> Result<()> {
        let mock_response = Bytes::from("001e# service=git-upload-pack
0000015523f0bc3b5c7c3108e41c448f01a3db31e7064bbb HEADmulti_ack thin-pack side-band side-band-64k ofs-delta shallow deepen-since deepen-not deepen-relative no-progress include-tag multi_ack_detailed allow-tip-sha1-in-want allow-reachable-sha1-in-want no-done symref=HEAD:refs/heads/master filter object-format=sha1 agent=git/github-0ecc5b5f94fa
003f23f0bc3b5c7c3108e41c448f01a3db31e7064bbb refs/heads/master
0000");
        let expected_ref =
            GitRef::new("003f", "23f0bc3b5c7c3108e41c448f01a3db31e7064bbb", "master");
        let expected_result = vec![expected_ref];
        let result = process_ref_discovery_response(&mock_response.slice(34..))?;

        assert_eq!(result, expected_result);

        Ok(())
    }

    #[test]
    fn should_extract_multiple_refs() -> Result<()> {
        let mock_response = Bytes::from("001e# service=git-upload-pack
00000155cb13b1d4e0751da3f6a3e0ba9ca9c61b9a1ee41f HEADmulti_ack thin-pack side-band side-band-64k ofs-delta shallow deepen-since deepen-not deepen-relative no-progress include-tag multi_ack_detailed allow-tip-sha1-in-want allow-reachable-sha1-in-want no-done symref=HEAD:refs/heads/master filter object-format=sha1 agent=git/github-84a1a651248e
0055f995bad1cf42515e59934d0c24194402b5ea6e65 refs/heads/attempting_to_make_an_editor
004951514685f102183cfa64df603560351a817b5093 refs/heads/chapter2_command
003fcb13b1d4e0751da3f6a3e0ba9ca9c61b9a1ee41f refs/heads/master
003e9970a007659cd9f286f5e91e8dd3a6873979aabf refs/pull/1/head
003f92af60e756e49184c25690f067a1c380f3b9e8a3 refs/pull/10/head
0000");
        let expected_refs = vec![
            GitRef::new(
                "0055",
                "f995bad1cf42515e59934d0c24194402b5ea6e65",
                "attempting_to_make_an_editor",
            ),
            GitRef::new(
                "0049",
                "51514685f102183cfa64df603560351a817b5093",
                "chapter2_command",
            ),
            GitRef::new("003f", "cb13b1d4e0751da3f6a3e0ba9ca9c61b9a1ee41f", "master"),
        ];
        let expected_result = expected_refs;
        let result = process_ref_discovery_response(&mock_response.slice(34..))?;

        assert_eq!(result, expected_result);

        Ok(())
    }
}
