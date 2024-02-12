use anyhow::Context;
use anyhow::{bail, Result};
use bytes::Bytes;
use flate2::read::ZlibDecoder;
use reqwest::get;
use std::collections::HashMap;
use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::path::Path;
use std::path::PathBuf;

use crate::init;
use crate::process_packfile::{apply_delta_instruction, ObjectType};
use crate::process_packfile::{read_size_encoding, read_type_and_size};
use crate::utils::create_directory;
use crate::utils::save_to_disk;

pub async fn clone(uri: &str, target_dir: &str) -> Result<()> {
    // create directory
    let target_directory = Path::new(".").join(target_dir);

    create_directory(&target_directory).context("create directory")?;
    init::init(target_directory.clone());

    let refs = discover_references(uri)
        .await
        .context("discovering references")?;
    let commit = get_commit(&refs.last().expect("getting commit bytes").commit_hash, uri)
        .await
        .context("getting commit")?;
    let pack_file = Packfile::new(commit.slice(..21)).context("creating Packfile metadata")?;
    let mut cursed_packfile = Cursor::new(&commit[20..]);
    let mut git_objects = HashMap::new();

    for _ in 0..pack_file.object_count {
        let object_type =
            read_type_and_size(&mut cursed_packfile).context("reading type and size")?;

        let _hash = match &object_type {
            ObjectType::Commit(size) => handle_normal_object_type(
                *size,
                "commit",
                &mut cursed_packfile,
                target_directory.clone(),
                &mut git_objects,
            )
            .context("handling normal object type as commit")?,
            ObjectType::Tree(size) => handle_normal_object_type(
                *size,
                "tree",
                &mut cursed_packfile,
                target_directory.clone(),
                &mut git_objects,
            )
            .context("handling normal object type as tree")?,
            ObjectType::Blob(size) => handle_normal_object_type(
                *size,
                "blob",
                &mut cursed_packfile,
                target_directory.clone(),
                &mut git_objects,
            )
            .context("handling normal object type as blob")?,
            ObjectType::Tag(size) => handle_normal_object_type(
                *size,
                "tag",
                &mut cursed_packfile,
                target_directory.clone(),
                &mut git_objects,
            )
            .context("handling normal object type as tag")?,
            ObjectType::OfsDelta(_) => {
                handle_ofs_delta();
                None
            }
            ObjectType::RefDelta(size) => {
                handle_ref_delta(&mut cursed_packfile, &git_objects)
                    .await
                    .context("handling ref/hash delta")?;
                None
            }
            ObjectType::Unknown => unreachable!(),
        };
    }

    Ok(())
}

// Implement using delta instructions at https://dev.to/calebsander/git-internals-part-2-packfiles-1jg8
fn handle_ofs_delta() {
    panic!("attempting to handle ofs delta");
}

async fn handle_ref_delta<R: Read + AsRef<[u8]>>(
    mut cursed_packfile: &mut Cursor<R>,
    git_objects: &HashMap<Vec<u8>, Vec<u8>>,
) -> Result<()> {
    let mut hash = [0; 20];

    cursed_packfile
        .read(&mut hash)
        .context("reading hash from cursed packfile")?;

    let _current_cursed_packfile_position = cursed_packfile.position();
    let mut decoder = ZlibDecoder::new(&mut cursed_packfile);
    let base_object_size = read_size_encoding(&mut decoder).context("reading base object size")?;
    let new_object_size = read_size_encoding(&mut decoder).context("reading new object size")?;
    let base = git_objects
        .get(&hash.to_vec())
        .expect("don't have git object");
    let mut decompressed_object = Vec::with_capacity(new_object_size);

    // decoder.read_to_end(&mut decompressed_object)?;

    // let count = decoder.total_in();

    // cursed_packfile.seek(std::io::SeekFrom::Start(
    //     current_cursed_packfile_position + count,
    // ))?;

    loop {
        let is_more = apply_delta_instruction(&mut decoder, &base, &mut decompressed_object)
            .context("applying delta instruction")?;

        if !is_more {
            break;
        }
    }

    dbg!(String::from_utf8(decompressed_object.to_owned())?);

    panic!();

    // let hash = hex::encode(hash);
    // let packfile_bytes = get_commit(&hash, repo_uri)
    //     .await
    //     .context("getting commit from cursor")?;
    // let _pack_file_meta =
    //     Packfile::new(packfile_bytes.slice(..21)).context("creating packfile metadata")?;
    // let mut packfile_cursor = Cursor::new(&packfile_bytes[20..]);
    // let type_and_size =
    //     read_type_and_size(&mut packfile_cursor).context("reading type and size")?;

    // let mut blob = Vec::with_capacity(type_and_size.get_size().expect("missing size"));

    // packfile_cursor
    //     .read(&mut blob)
    //     .context("reading blob data")?;

    Ok(())
}

fn handle_normal_object_type<R: Read + AsRef<[u8]>>(
    size: usize,
    object_type: &str,
    mut cursed_packfile: &mut Cursor<R>,
    target_directory: PathBuf,
    git_objects: &mut HashMap<Vec<u8>, Vec<u8>>,
) -> Result<Option<Vec<u8>>> {
    let current_cursed_packfile_position = cursed_packfile.position();
    let mut decompressed_object = Vec::with_capacity(size);
    let mut decoder = ZlibDecoder::new(&mut cursed_packfile);
    decoder.read_to_end(&mut decompressed_object)?;
    let count = decoder.total_in();

    cursed_packfile.seek(std::io::SeekFrom::Start(
        current_cursed_packfile_position + count,
    ))?;
    let header = format!("{object_type} {size}\0",);
    let mut commit = header.into_bytes();
    commit.extend(decompressed_object);

    let hash = save_to_disk(&commit, target_directory)?;

    git_objects.insert(hash.clone(), commit);

    Ok(Some(hash))
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
#[allow(dead_code)]
struct Packfile {
    head: String,
    signature: String,
    version: u32,
    object_count: u32,
}

impl Packfile {
    pub fn new(commit: Bytes) -> Result<Self> {
        let head = std::str::from_utf8(&commit[0..8])?.to_owned();
        let signature = std::str::from_utf8(&commit[8..12])?.to_owned();
        let version = u32::from_be_bytes(commit[12..16].try_into()?);
        let object_count = u32::from_be_bytes(commit[16..20].try_into()?);

        Ok(Self {
            head,
            signature,
            version,
            object_count,
        })
    }
}

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
