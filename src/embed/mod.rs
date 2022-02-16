//! This module provides an in-memory filesystem implementation.

mod linked_storage;
pub use linked_storage::LinkedStorage;

use crate::Vfd;
use std::{collections::HashMap, path::Path};

pub(crate) trait NodeIdTrait {
    fn ino(&self) -> u64;
}

pub(crate) trait NodeFileBody {
    fn content(&self) -> &[u8];
}

pub(crate) struct DirEntry<S: Storage + ?Sized> {
    pub(crate) name: String,
    pub(crate) link_id: S::LinkId,
}

impl<S: Storage> Clone for DirEntry<S> {
    fn clone(&self) -> Self {
        DirEntry {
            name: self.name.clone(),
            link_id: self.link_id,
        }
    }
}

pub(crate) trait NodeDirBody<S: Storage + ?Sized> {
    type Iter: Iterator<Item = DirEntry<S>>;
    fn entries(&self) -> Self::Iter;
}

/// A storage that can be used to store files and directories.
pub(crate) trait Storage {
    type NodeId: NodeIdTrait + Clone + Copy;
    type LinkId: Clone + Copy;
    type NodeFileBody: NodeFileBody;
    type NodeDirBody: NodeDirBody<Self>;

    /// Creates a new root node.
    fn new_root_dir(&mut self) -> (Self::NodeId, Self::LinkId);

    /// Creates a new directory node under the given parent node.
    fn new_dir(
        &mut self,
        parent: (Self::NodeId, Self::LinkId),
        name: String,
    ) -> (Self::NodeId, Self::LinkId);

    /// Creates a new file node under the given parent node.
    fn new_file(
        &mut self,
        parent: (Self::NodeId, Self::LinkId),
        name: String,
        content: Vec<u8>,
    ) -> (Self::NodeId, Self::LinkId);

    /// Resolve a node from its id.
    fn get_inode(&self, node_id: &Self::NodeId) -> Node<Self>;

    /// Resolve a link from its id.
    fn get_link(&self, link_id: &Self::LinkId) -> Link<Self>;

    /// Resolve a node from base node and relative path.
    fn resolve_node(
        &self,
        base: Self::NodeId,
        base_link: Self::LinkId,
        path: &Path,
    ) -> Result<(Self::NodeId, Self::LinkId), wasi::Errno>;
}

pub(crate) enum Node<'a, S: Storage + ?Sized> {
    File(&'a S::NodeFileBody),
    Dir(&'a S::NodeDirBody),
}

/// Represent a hard link to an inode
pub(crate) struct Link<S: Storage + ?Sized> {
    pub(crate) parent: Option<S::LinkId>,
    pub(crate) node: S::NodeId,
}

impl<S: Storage> Clone for Link<S> {
    fn clone(&self) -> Self {
        Link {
            parent: self.parent,
            node: self.node,
        }
    }
}

impl<S: Storage> Copy for Link<S> {}

pub(crate) struct FdEntry<S: Storage + ?Sized> {
    pub(crate) offset: usize,
    pub(crate) link_id: S::LinkId,
    pub(crate) node_id: S::NodeId,
    pub(crate) flags: wasi::Fdflags,
}

pub(crate) struct PreopenedDir {
    pub(crate) path: String,
}

pub(crate) struct EmbeddedFs<S: Storage> {
    preopened_dirs: Vec<PreopenedDir>,
    storage: S,

    opens: HashMap<Vfd, FdEntry<S>>,
    fd_issuer: IdIssuer<Vfd>,
}

#[derive(Default)]
struct IdIssuer<Id> {
    next_id: Id,
}

impl<Id> IdIssuer<Id> {
    fn new(base: Id) -> Self {
        Self { next_id: base }
    }
}

impl<Id: std::ops::AddAssign<u32> + Clone> IdIssuer<Id> {
    fn issue(&mut self) -> Id {
        let id = self.next_id.clone();
        self.next_id += 1;
        id
    }
}

impl<S: Storage> Default for EmbeddedFs<S>
where
    S: Default,
{
    fn default() -> Self {
        Self::new(S::default())
    }
}

impl<S: Storage> EmbeddedFs<S> {
    pub(crate) fn new(storage: S) -> Self {
        Self {
            preopened_dirs: vec![],
            storage,
            opens: HashMap::new(),
            fd_issuer: IdIssuer::new(0_u32),
        }
    }

    pub(crate) fn preopen_dir(&mut self, path: String) -> (Vfd, S::NodeId, S::LinkId) {
        assert!(self.preopened_dirs.len() == self.opens.len());
        let fd = self.fd_issuer.issue();
        self.preopened_dirs.push(PreopenedDir { path });
        let (node_id, link_id) = self.storage.new_root_dir();
        self.opens.insert(
            fd,
            FdEntry {
                offset: 0,
                node_id,
                link_id,
                flags: 0,
            },
        );
        (fd, node_id, link_id)
    }

    pub(crate) fn get_preopened_dir_path(&self, vfd: Vfd) -> Option<&str> {
        let vfd = vfd as usize;
        if vfd >= self.preopened_dirs.len() {
            return None;
        }
        Some(&self.preopened_dirs[vfd].path)
    }

    pub(crate) fn create_file(
        &mut self,
        dir_node: S::NodeId,
        dir_link: S::LinkId,
        mut relpath: &str,
        content: Vec<u8>,
    ) -> Result<(), u16> {
        let mut cursor = match self.storage.get_inode(&dir_node) {
            Node::Dir { .. } => (dir_node, dir_link),
            _ => return Err(wasi::ERRNO_BADF.raw()),
        };
        if relpath.starts_with('/') {
            relpath = &relpath[1..];
        }
        let components = relpath.split('/').collect::<Vec<_>>();
        let filename = match components.last() {
            Some(filename) => *filename,
            None => return Err(wasi::ERRNO_NOENT.raw()),
        };

        let components_len = components.len();
        'find_parent_node: for component in components.into_iter().take(components_len - 1) {
            if component == "." {
                continue;
            }
            let entries = match self.storage.get_inode(&cursor.0) {
                Node::Dir(body) => body.entries(),
                _ => return Err(wasi::ERRNO_BADF.raw()),
            };
            for entry in entries {
                if component == entry.name {
                    cursor = (self.storage.get_link(&entry.link_id).node, entry.link_id);
                    continue 'find_parent_node;
                }
            }
            // create a new intermediate directory
            {
                let (new_dir_id, new_link_id) = self.storage.new_dir(cursor, component.to_string());
                cursor = (new_dir_id, new_link_id);
            }
        }

        self.storage.new_file(cursor, filename.to_string(), content);

        Ok(())
    }

    pub(crate) fn get_node_id_by_link(&self, id: S::LinkId) -> S::NodeId {
        self.storage.get_link(&id).node
    }

    pub(crate) fn get_node(&self, fd: Vfd) -> Result<Node<S>, wasi::Errno> {
        match self.opens.get(&fd) {
            Some(entry) => Ok(self.storage.get_inode(&entry.node_id)),
            None => Err(wasi::ERRNO_BADF),
        }
    }

    pub(crate) fn get_fd_stat(&self, fd: Vfd) -> Result<wasi::Fdstat, wasi::Errno> {
        const READ_ONLY_RIGHTS: wasi::Rights = wasi::RIGHTS_FD_READ
            | wasi::RIGHTS_FD_ADVISE
            | wasi::RIGHTS_PATH_OPEN
            | wasi::RIGHTS_FD_READDIR
            | wasi::RIGHTS_FD_FILESTAT_GET;
        let entry = match self.opens.get(&fd) {
            Some(entry) => entry,
            None => return Err(wasi::ERRNO_BADF),
        };
        Ok(match self.storage.get_inode(&entry.node_id) {
            Node::File { .. } => wasi::Fdstat {
                fs_filetype: wasi::FILETYPE_REGULAR_FILE,
                fs_flags: entry.flags,
                fs_rights_base: READ_ONLY_RIGHTS,
                fs_rights_inheriting: READ_ONLY_RIGHTS,
            },
            Node::Dir { .. } => wasi::Fdstat {
                fs_filetype: wasi::FILETYPE_DIRECTORY,
                fs_flags: entry.flags,
                fs_rights_base: READ_ONLY_RIGHTS,
                fs_rights_inheriting: READ_ONLY_RIGHTS,
            },
        })
    }

    pub(crate) fn get_filestat_from_node_id(&self, node_id: S::NodeId) -> wasi::Filestat {
        let mut stat = wasi::Filestat {
            dev: Default::default(),
            ino: Default::default(),
            filetype: wasi::FILETYPE_UNKNOWN,
            nlink: Default::default(),
            size: Default::default(),
            atim: Default::default(),
            mtim: Default::default(),
            ctim: Default::default(),
        };
        stat.ino = node_id.ino();
        match self.storage.get_inode(&node_id) {
            Node::File(body) => {
                stat.filetype = wasi::FILETYPE_REGULAR_FILE;
                stat.size = body.content().len() as u64;
                stat
            }
            Node::Dir { .. } => {
                stat.filetype = wasi::FILETYPE_DIRECTORY;
                stat
            }
        }
    }

    pub(crate) fn get_fd_entry_mut(&mut self, fd: Vfd) -> Result<&mut FdEntry<S>, wasi::Errno> {
        match self.opens.get_mut(&fd) {
            Some(open_file) => Ok(open_file),
            None => Err(wasi::ERRNO_BADF),
        }
    }

    pub(crate) fn get_fd_entry(&self, fd: Vfd) -> Result<&FdEntry<S>, wasi::Errno> {
        match self.opens.get(&fd) {
            Some(open_file) => Ok(open_file),
            None => Err(wasi::ERRNO_BADF),
        }
    }

    pub(crate) fn close_file(&mut self, fd: Vfd) -> Result<(), wasi::Errno> {
        match self.opens.remove(&fd) {
            Some(_) => Ok(()),
            None => Err(wasi::ERRNO_BADF),
        }
    }

    pub(crate) fn open_file(
        &mut self,
        base: Vfd,
        path: &Path,
        fdflags: wasi::Fdflags,
    ) -> Result<Vfd, wasi::Errno> {
        let base = &self.opens[&base];
        let (node_id, link_id) = self
            .storage
            .resolve_node(base.node_id, base.link_id, path)?;
        let new_fd = self.fd_issuer.issue();
        self.opens.insert(
            new_fd,
            FdEntry {
                offset: 0,
                node_id,
                link_id,
                flags: fdflags,
            },
        );
        Ok(new_fd)
    }

    pub(crate) fn get_filestat_at_path(
        &self,
        base: Vfd,
        path: &Path,
    ) -> Result<wasi::Filestat, wasi::Errno> {
        let base = &self.opens[&base];
        let (node_id, _) = self
            .storage
            .resolve_node(base.node_id, base.link_id, path)?;
        let res = self.get_filestat_from_node_id(node_id);
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{EmbeddedFs, LinkedStorage};

    #[test]
    fn test_embedded_node_create_file() {
        let content = "Hello".as_bytes().to_vec();
        let mut fs = EmbeddedFs::<LinkedStorage>::default();
        let (_, node_id, link_id) = fs.preopen_dir("/".to_string());
        fs.create_file(node_id, link_id, "hello.txt", content)
            .unwrap();
    }

    #[test]
    fn test_get_filestat_at_path_for_non_existing() {
        let mut fs = EmbeddedFs::<LinkedStorage>::default();
        let (vfd, _, _) = fs.preopen_dir("/".to_string());
        let result = fs.get_filestat_at_path(vfd, Path::new("/not-exist"));
        assert!(result.is_err());
    }
}
