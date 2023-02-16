use std::fs;

use pickledb::PickleDb;
use serenity::model::webhook::{Webhook};

use crate::{errors::SipError, fetcher::SipPost};

// u isto vreme i vrsi validaciju
pub fn setup_storage(database: &mut PickleDb) -> Result<(), SipError> {
    // logs
    fs::create_dir_all("./logs").map_err(|err| SipError::FileSystemError(err.to_string()))?;

    // hook
    if !database.exists("sip_hooks") {
        database.set("sip_hooks", &Vec::<Webhook>::new()).map_err(|err| SipError::StorageError(err.to_string()))?;
    }

    // levi - najnovije vesti
    if !database.exists("levi_stari") {
        database.set("levi_stari", &Vec::<SipPost>::new()).map_err(|err| SipError::StorageError(err.to_string()))?;
    }
    // if !database.exists("levi_novi") {
    //     database.set("levi_novi", &Vec::<SipPost>::new()).map_err(|err| SipError::StorageError(err.to_string()))?;
    // }

    // desni - vazna obavestenja
    if !database.exists("desni_stari") {
        database.set("desni_stari", &Vec::<SipPost>::new()).map_err(|err| SipError::StorageError(err.to_string()))?;
    }
    // if !database.exists("desni_novi") {
    //     database.set("desni_novi", &Vec::<SipPost>::new()).map_err(|err| SipError::StorageError(err.to_string()))?;
    // }

    return Ok(());
}