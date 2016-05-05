#pragma once

#include <stdbool.h>

struct file_watcher;

struct file_watcher *
file_watcher_alloc(const char *pathname, bool debug);

void
file_watcher_free(struct file_watcher *fw);

/**
 * Blocking
 * Returns 0 if write came or -1 if file was deleted
 */
int
file_watcher_wait_write(struct file_watcher *fw);
