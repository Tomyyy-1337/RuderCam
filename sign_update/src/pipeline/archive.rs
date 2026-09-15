use std::{fs::File, io, path::Path, sync::mpsc::Sender};

use tar::{Builder, Header};

use crate::events::AppEvent;

use super::{BACKEND_BIN_PATH, BACKEND_CODE_PATH, STATIC_DIR};

pub fn create_archive(
    tx: &Sender<AppEvent>,
    version: &str,
    combined_hash: &[u8],
) -> io::Result<()> {
    let _ = tx.send(AppEvent::Output(
        "\x1b[36mCreating update archive...\x1b[0m".into(),
    ));

    let file = File::create("update.tar")?;
    let mut archive = Builder::new(file);

    archive.append_dir_all("static", STATIC_DIR)?;
    archive.append_file(
        "server",
        &mut File::open(Path::new(BACKEND_CODE_PATH).join(BACKEND_BIN_PATH))?,
    )?;

    let mut version_header = Header::new_gnu();
    version_header.set_size(version.as_bytes().len() as u64);
    archive.append_data(&mut version_header, "version.txt", version.as_bytes())?;

    let mut hash_header = Header::new_gnu();
    hash_header.set_size(combined_hash.len() as u64);
    archive.append_data(&mut hash_header, "update_hash", combined_hash)?;

    archive.into_inner()?;

    let _ = tx.send(AppEvent::Output(
        "\x1b[32mArchive has been created successfully at ./update.tar\x1b[0m".into(),
    ));

    Ok(())
}
