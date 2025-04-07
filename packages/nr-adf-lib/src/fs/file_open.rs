use std::path::Path;

use crate::errors::*;

use super::amiga_dos::*;
use super::file::*;


#[derive(Clone, Debug, Default)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
}

impl OpenOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(
        &mut self,
        read: bool,
    ) -> &mut Self {
        self.read = read;
        self
    }

    pub fn write(
        &mut self,
        write: bool,
    ) -> &mut Self {
        if write {
            self.write = true;
        } else {
            self.append = false;
            self.truncate = false;
            self.write = false;
        }
        self
    }

    pub fn append(
        &mut self,
        append: bool,
    ) -> &mut Self {
        if append {
            self.append = true;
            self.write = true;
        } else {
            self.append = false;
        }
        self
    }

    pub fn truncate(
        &mut self,
        truncate: bool,
    ) -> &mut Self {
        if truncate {
            self.truncate = true;
            self.write = true;
        } else {
            self.truncate = false;
        }
        self
    }

    pub fn create(
        &mut self,
        create: bool,
    ) -> &mut Self {
        if create {
            self.create = create;
        } else {
            self.create = false;
            self.create_new = false;
        }
        self
    }

    pub fn create_new(
        &mut self,
        create_new: bool,
    ) -> &mut Self {
        if create_new {
            self.create = true;
            self.create_new = true;
        } else {
            self.create_new = false;
        }
        self
    }

    pub fn open<P: AsRef<Path>>(
        &self,
        fs: &AmigaDos,
        path: P,
    ) -> Result<File, Error> {
        if !(self.read || self.write || self.append) {
            return Err(Error::InvalidFileModeError);
        }

        if self.append && self.truncate {
            return Err(Error::InvalidFileModeError);
        }

        if (self.create || self.create_new) && !self.write {
            return Err(Error::InvalidFileModeError);
        }

        let mut mode: usize = 0;

        if self.read {
            mode = mode | FileMode::Read;
        }

        if self.write {
            mode = mode | FileMode::Write;
        }

        let mut file = if self.create {
            File::try_create(fs, path.as_ref(), mode, self.create_new)?
        } else {
            File::try_open(fs, path.as_ref(), mode)?
        };

        if self.truncate {
            file.set_len(0)?;
        }

        if self.append {
            file.pos = file.size;
        }

        Ok(file)
    }
}
