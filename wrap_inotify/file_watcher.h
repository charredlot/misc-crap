#pragma once

/**
 * thin wrapper around inotify
 */

#include <stdbool.h>

struct file_watcher;

struct file_watcher * file_watcher_alloc(const char *pathname, bool debug);

void file_watcher_free(struct file_watcher *fw);

/**
 * Blocks until next write. User should reallocate file_watcher and reopen the
 * file itself generally on -1 error
 *
 * @return
 *     0 if write or truncate happened
 *     -1 if file was deleted or other catastrophic event like unmount
 */
int file_watcher_wait_write(struct file_watcher *fw);
