#include <assert.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

typedef uint16_t wasi_errno_t;
#define WASI_ERRNO_SUCCESS (0)
#define WASI_ERRNO_NOENT (44)
#define WASI_ERRNO_NOTDIR (54)

struct wasi_vfs_embed_linked_storage {};

/*

# Data structure example

Consider the following case:
  /a
    ./b_file
    ./c_dir
      ./symlink_a  -> [symbolic link to /a]
      ./hardlink_a -> [hard link to /a]


           |-> NULL
           |
           |         |----------------------------------------------|
           |         |                                              |
           |         |                            |-----------------|-----|
           |         v                            v                 |     |
           |   |--------|            |-----dir node A-----|         |     |
           --- | link A | --[node]-> | - name: "c_dir"    |         |     |
           |-> |--------|            |   link: link C     | -----|  |     |
           |                         | - name: "b_file"   |      |  |     |
           |                         |   link: link B     | --|  |  |     |
        [parent]                     |--------------------|   |  |  |     |
           |                                                  |  |  |     |
           |         |----------------------------------------|  |  |     |
           |         v                                           |  |     |
           |   |--------|            |-------------|             |  |     |
           |-- | link B | --[node]-> | file node B |             |  |     |
           |   |--------|            |-------------|             |  |     |
           |                                                     |  |     |
           |         |-------------------------------------------|  |     |
           |         v                                              |     |
           |   |--------|            |-----dir node C-------|       |     |
           --- | link C | --[node]-> | - name: "symlink_a"  |       |     |
           |-> |--------|            |   link: link A       | ------|     |
           |                         | - name: "hardlink_a" |             |
           |                         |   link: link D       | ------|     |
           |                         |----------------------|       |     |
        [parent]                                                    |     |
           |         |----------------------------------------------|     |
           |         v                                                    |
           |   |--------|                                                 |
           |-- | link D | --[node]----------------------------------------|
               |--------|
*/

struct wasi_vfs_node;
struct wasi_vfs_link {
  struct wasi_vfs_link *parent;
  struct wasi_vfs_node *node;
};

struct wasi_vfs_dirent {
  struct wasi_vfs_link *link;
  const char *name;
  struct wasi_vfs_dirent *next;
};

// IMPORTANT: This layout must match the layout of struct InnerNode in
// Rust-side.
struct wasi_vfs_node {
  bool is_dir;
  size_t count;
  union {
    uint8_t *data;
    struct wasi_vfs_dirent *dirents;
  };
};

typedef struct {
  struct wasi_vfs_node *node;
  struct wasi_vfs_link *link;
} node_link_t;

static void insert_dirent(struct wasi_vfs_node *node,
                          struct wasi_vfs_dirent *dirent) {
  dirent->next = node->dirents;
  node->dirents = dirent;
  node->count++;
}

static struct wasi_vfs_node *new_node(bool is_dir) {
  struct wasi_vfs_node *node = malloc(sizeof(struct wasi_vfs_node));
  node->is_dir = is_dir;
  node->count = 0;
  node->data = NULL;
  return node;
}

static struct wasi_vfs_link *new_link(struct wasi_vfs_node *node) {
  struct wasi_vfs_link *link = malloc(sizeof(struct wasi_vfs_link));
  link->parent = NULL;
  link->node = node;
  return link;
}

static struct wasi_vfs_dirent *new_dirent(struct wasi_vfs_link *link,
                                          const char *name) {
  struct wasi_vfs_dirent *dirent = malloc(sizeof(struct wasi_vfs_dirent));
  dirent->link = link;
  dirent->name = strdup(name);
  dirent->next = NULL;
  return dirent;
}

struct wasi_vfs_embed_linked_storage *wasi_vfs_embed_linked_storage_new(void) {
  return malloc(sizeof(struct wasi_vfs_embed_linked_storage));
}

void wasi_vfs_embed_linked_storage_free(
    struct wasi_vfs_embed_linked_storage *self) {
  free(self);
}

node_link_t wasi_vfs_embed_linked_storage_preopen_new_dir(
    struct wasi_vfs_embed_linked_storage *self) {
  (void)self;
  struct wasi_vfs_node *node = new_node(true);
  struct wasi_vfs_link *link = new_link(node);
  return (node_link_t){node, link};
}

node_link_t wasi_vfs_embed_linked_storage_new_dir(
    struct wasi_vfs_embed_linked_storage *self, const node_link_t *parent,
    char *name) {
  (void)self;
  struct wasi_vfs_node *node = new_node(true);
  struct wasi_vfs_link *link = new_link(node);
  link->parent = parent->link;

  assert(parent->node->is_dir && "parent is not a dir");

  struct wasi_vfs_dirent *dirent = new_dirent(link, name);
  insert_dirent(parent->node, dirent);

  return (node_link_t){node, link};
}

node_link_t wasi_vfs_embed_linked_storage_new_file(
    struct wasi_vfs_embed_linked_storage *self, const node_link_t *parent,
    char *name, uint8_t *content, size_t content_len) {

  (void)self;
  struct wasi_vfs_node *node = new_node(false);
  node->count = content_len;
  node->data = content;

  struct wasi_vfs_link *link = new_link(node);
  link->parent = parent->link;
  link->node = node;

  struct wasi_vfs_dirent *dirent = new_dirent(link, name);
  insert_dirent(parent->node, dirent);

  return (node_link_t){node, link};
}

wasi_errno_t wasi_vfs_embed_linked_storage_resolve_node_at(
    struct wasi_vfs_embed_linked_storage *self, const node_link_t *base,
    const char *path, node_link_t *out) {
  (void)self;

  node_link_t current = *base;

find_parent_node:
  while (path[0] != '\0') {
    // strip leading '/'
    while (path[0] == '/') {
      path++;
    }
    const char *component = path;
    while (path[0] != '/' && path[0] != '\0') {
      path++;
    }
    const size_t component_len = path - component;

    // expect that the current node is a dir
    if (!current.node->is_dir) {
      return WASI_ERRNO_NOTDIR;
    }

    // ok we are in a dir
    if (component_len == 1 && component[0] == '.') {
      // empty component, skip
      continue;
    }

    if (component_len == 2 && component[0] == '.' && component[1] == '.') {
      // '..' component, go up
      if (current.link->parent == NULL) {
        // we are already at the root
        return WASI_ERRNO_NOTDIR;
      }
      struct wasi_vfs_link *parent = current.link->parent;
      current = (node_link_t){.node = parent->node, .link = parent};
      continue;
    }

    // ok we have flattened special components, find children
    struct wasi_vfs_dirent *dirent = current.node->dirents;
    while (dirent != NULL) {
      if (strncmp(dirent->name, component, component_len) == 0 &&
          dirent->name[component_len] == '\0') {
        // found the child
        current =
            (node_link_t){.node = dirent->link->node, .link = dirent->link};
        goto find_parent_node;
      }
      dirent = dirent->next;
    }
    return WASI_ERRNO_NOENT;
  }
  *out = current;
  return WASI_ERRNO_SUCCESS;
}
