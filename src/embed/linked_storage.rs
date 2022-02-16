//! This module consists of C and Rust implementations.
//! C part builds link graph by chaining nodes by pointers.
//! Rust part is a thin wrapper to conform to `Storage` trait.

use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
    path::Path,
};

use super::{DirEntry, Link, Node, NodeDirBody, NodeFileBody, NodeIdTrait, Storage};

#[repr(transparent)]
#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub struct NodeId(*const std::ffi::c_void);

impl NodeIdTrait for NodeId {
    fn ino(&self) -> u64 {
        self.0 as u64
    }
}

#[repr(transparent)]
#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub struct LinkId(*const std::ffi::c_void);

pub struct LinkedStorage {
    context: *mut std::ffi::c_void,
}

#[repr(C)]
struct NodeLink {
    node_id: NodeId,
    link_id: LinkId,
}

impl NodeFileBody for InnerNode {
    fn content(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(std::mem::transmute(self.dir_or_file), self.count) }
    }
}

impl NodeDirBody<LinkedStorage> for InnerNode {
    type Iter = LinkedStorageIterator;
    fn entries(&self) -> Self::Iter {
        LinkedStorageIterator(self.dir_or_file as *const InnerDirent)
    }
}

pub(crate) struct LinkedStorageIterator(*const InnerDirent);
impl Iterator for LinkedStorageIterator {
    type Item = DirEntry<LinkedStorage>;

    fn next(&mut self) -> Option<DirEntry<LinkedStorage>> {
        if self.0.is_null() {
            return None;
        }
        let (entry, next) = unsafe {
            let node = &(*self.0);
            let name = CStr::from_ptr(node.name).to_string_lossy().into_owned();
            let link_id = LinkId(node.link_id);
            (DirEntry::<LinkedStorage> { name, link_id }, node.next)
        };
        self.0 = next;
        Some(entry)
    }
}

#[repr(C)]
pub(crate) struct InnerNode {
    is_dir: bool,
    count: usize,
    dir_or_file: *const std::ffi::c_void,
}

#[repr(C)]
struct InnerDirent {
    link_id: *const std::ffi::c_void,
    name: *const i8,
    next: *const InnerDirent,
}

#[repr(C)]
struct InnerLink {
    parent: LinkId,
    node: NodeId,
}

extern "C" {
    fn wasi_vfs_embed_linked_storage_new() -> *mut std::ffi::c_void;
    fn wasi_vfs_embed_linked_storage_free(context: *mut std::ffi::c_void);
    fn wasi_vfs_embed_linked_storage_preopen_new_dir(context: *mut std::ffi::c_void) -> NodeLink;
    fn wasi_vfs_embed_linked_storage_new_dir(
        context: *mut std::ffi::c_void,
        parent: *const NodeLink,
        name: *const i8,
    ) -> NodeLink;
    fn wasi_vfs_embed_linked_storage_new_file(
        context: *mut std::ffi::c_void,
        parent: *const NodeLink,
        name: *const i8,
        content: *const u8,
        content_len: usize,
    ) -> NodeLink;
    fn wasi_vfs_embed_linked_storage_resolve_node_at(
        context: *mut std::ffi::c_void,
        base: *const NodeLink,
        path: *const i8,
        ret: *mut NodeLink,
    ) -> wasi::Errno;
}

impl Default for LinkedStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl LinkedStorage {
    pub(crate) fn new() -> Self {
        Self {
            context: unsafe { wasi_vfs_embed_linked_storage_new() },
        }
    }
}

impl Storage for LinkedStorage {
    type NodeId = NodeId;
    type LinkId = LinkId;
    type NodeFileBody = InnerNode;
    type NodeDirBody = InnerNode;

    fn new_root_dir(&mut self) -> (NodeId, LinkId) {
        unsafe {
            let ret = wasi_vfs_embed_linked_storage_preopen_new_dir(self.context);
            (ret.node_id, ret.link_id)
        }
    }

    fn new_dir(&mut self, parent: (NodeId, LinkId), name: String) -> (NodeId, LinkId) {
        unsafe {
            let name = CString::new(name).unwrap();
            let link = NodeLink {
                node_id: parent.0,
                link_id: parent.1,
            };
            let result = wasi_vfs_embed_linked_storage_new_dir(self.context, &link, name.as_ptr());
            (result.node_id, result.link_id)
        }
    }

    fn new_file(
        &mut self,
        parent: (NodeId, LinkId),
        name: String,
        content: Vec<u8>,
    ) -> (NodeId, LinkId) {
        unsafe {
            let name = CString::new(name).unwrap();
            let content_len = content.len();
            let content = Box::new(content);
            let link = NodeLink {
                node_id: parent.0,
                link_id: parent.1,
            };
            let result = wasi_vfs_embed_linked_storage_new_file(
                self.context,
                &link,
                name.as_ptr(),
                content.leak().as_ptr(),
                content_len,
            );
            (result.node_id, result.link_id)
        }
    }

    fn get_inode(&self, node_id: &NodeId) -> Node<Self> {
        unsafe {
            let node = node_id.0 as *const InnerNode;
            if (*node).is_dir {
                Node::Dir(node.as_ref().unwrap())
            } else {
                Node::File(node.as_ref().unwrap())
            }
        }
    }

    fn get_link(&self, link_id: &LinkId) -> Link<Self> {
        unsafe {
            let inner_link = link_id.0 as *const InnerLink;
            let inner_link = inner_link.as_ref().unwrap();
            let parent = if inner_link.parent.0.is_null() {
                None
            } else {
                Some(inner_link.parent)
            };
            Link {
                parent,
                node: inner_link.node,
            }
        }
    }

    fn resolve_node(
        &self,
        base: NodeId,
        base_link: LinkId,
        path: &Path,
    ) -> Result<(NodeId, LinkId), wasi::Errno> {
        unsafe {
            let path: &[i8] = std::mem::transmute(path.as_os_str());
            let link = NodeLink {
                node_id: base,
                link_id: base_link,
            };
            let mut ret = MaybeUninit::uninit();
            let errno = wasi_vfs_embed_linked_storage_resolve_node_at(
                self.context,
                &link,
                path.as_ptr(),
                ret.as_mut_ptr(),
            );
            if errno == wasi::ERRNO_SUCCESS {
                let ret = ret.assume_init();
                Ok((ret.node_id, ret.link_id))
            } else {
                Err(errno)
            }
        }
    }
}

impl Drop for LinkedStorage {
    fn drop(&mut self) {
        unsafe { wasi_vfs_embed_linked_storage_free(self.context) }
    }
}
