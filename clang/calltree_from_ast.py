#!/usr/bin/python

import json
import os
import sys

from clang.cindex import TranslationUnit, CursorKind, TypeKind, Index, conf
import argparse


class Decl(object):
    def __init__(self, name):
        self.name = name
        self.callers = dict()
        self.calls = dict()

    def add_call(self, decl):
        self.calls[decl.name] = decl

    def add_caller(self, decl):
        self.callers[decl.name] = decl

    def print_calls(self, args):
        self.print_tree(lambda node: node.calls, args)

    def print_callers(self, args):
        self.print_tree(lambda node: node.callers, args)

    def print_tree(self, get_children, args):
        l = [self.name]

        depth = args.depth
        leaves = args.leaves
        prune = args.prune

        # format shamelessly cribbed from asciitree
        def recurse_print(node, children,
                          lines, parent_set, indent, curr_depth):
            length = len(children)
            for i, (name, child_node) in enumerate(children.items()):
                grandchildren = get_children(child_node)
                if leaves and grandchildren:
                    parent_set.add(name)
                    recurse_print(child_node, grandchildren,
                                  lines, parent_set, indent,
                                  curr_depth + 1)
                    continue

                if name in parent_set:
                    # don't infinite loop for e.g. recursive functions
                    if not leaves:
                        lines.append("{}+-- {}**".format(indent, name))
                    continue

                lines.append("{}+-- {}".format(indent, name))
                parent_set.add(name)

                if not grandchildren:
                    continue

                if depth is not None and curr_depth >= depth:
                    continue

                if i + 1 == length:
                    new_indent = indent + "    "
                else:
                    new_indent = indent + "|   "

                if prune:
                    new_set = parent_set
                else:
                    new_set = parent_set.copy()

                recurse_print(child_node, grandchildren,
                              lines, new_set, new_indent,
                              curr_depth + 1)

        recurse_print(self, get_children(self), l, set(), "", 0)
        print('\n'.join(l))

    def __str__(self):
        return "{}".format(self.name)


class CallInfo(object):
    def __init__(self):
        self.decls = dict()
        self._globals = dict()

    def add_global(self, cursor):
        self._globals[cursor.get_usr()] = cursor

    def add_decl(self, cursor):
        name = cursor.spelling
        return self.decls.setdefault(name, Decl(name))

    def get_or_create_decl(self, name):
        decl = self.decls.get(name)
        if not decl:
            decl = Decl(name)
            self.decls[name] = decl
        return decl

    def walk_decl(self, cursor, decl):
        call_func = None
        if ((cursor.kind == CursorKind.CALL_EXPR) or
           (cursor.kind == CursorKind.DECL_REF_EXPR and
            (cursor.type.kind == TypeKind.FUNCTIONNOPROTO or
             cursor.type.kind == TypeKind.FUNCTIONPROTO))):
            call_func = self.add_decl(cursor)
        elif cursor.kind == CursorKind.DECL_REF_EXPR:
            # this is for if you stick a function ptr in a global variable
            # this probably doesn't catch every case
            parent = cursor.referenced.canonical
            match = self._globals.get(parent.get_usr())
            if match:
                call_func = self.add_decl(match)

        if call_func and call_func.name:
            # TODO: locals won't have a cursor.spelling..should probably
            # figure out a better way to find if a cursor is a local var
            call_func.add_caller(decl)
            decl.add_call(call_func)

        for child in cursor.get_children():
            self.walk_decl(child, decl)

    def dump(self, filename):
        sys.stderr.write("saving output to {}\n".format(filename))
        sys.stderr.flush()
        calls = {name: list(decl.calls.keys())
                 for name, decl in self.decls.items()}
        callers = {name: list(decl.callers.keys())
                   for name, decl in self.decls.items()}
        with open(filename, "w") as f:
            json.dump({'callers': callers, "calls": calls}, f)

    @classmethod
    def load(cls, filename):
        sys.stderr.write("loading input from {}\n".format(filename))
        sys.stderr.flush()
        with open(filename, "r") as f:
            d = json.load(f)

        calls = d.get('calls')
        callers = d.get('callers')
        if not callers:
            raise Exception("No 'calls' or 'callers' object found")

        ci = CallInfo()
        for name, decl_callers in callers.items():
            # might have been created by a previous decl who calls this func
            decl = ci.get_or_create_decl(name)
            for caller_name in decl_callers:
                caller_decl = ci.get_or_create_decl(caller_name)
                decl.add_caller(caller_decl)
                ci.decls[caller_name] = caller_decl

            decl_calls = calls.get(name)
            if decl_calls:
                for call_name in decl_calls:
                    call_decl = ci.get_or_create_decl(call_name)
                    decl.add_call(call_decl)
        return ci


def _ast_files(d):
    for root, _, files in os.walk(d):
        for f in files:
            if not f.endswith('.ast'):
                continue
            yield os.path.join(root, f)


def _ast_files_to_callinfo(directory):
    index = Index(conf.lib.clang_createIndex(False, True))

    # don't list comprehend so we can get better error reporting
    units = []
    for path in _ast_files(directory):
        try:
            units.append((os.path.abspath(path),
                          TranslationUnit.from_ast_file(path, index)))
        except Exception as e:
            print("error parsing {}, python clang version might be different"
                  "from compiled clang version?".format(path))
            print(e.args)
            print(e.message)
            raise

    ci = CallInfo()

    for path, tu in units:
        for cursor in tu.cursor.get_children():
            # seems hacky, probably misses c++ cases
            # stuff from includes has the include's filename
            if ((cursor.kind == CursorKind.VAR_DECL) and
                (cursor.location.file.name == tu.spelling)):
                ci.add_global(cursor)

    for path, tu in units:
        # WARNING: this will fail silently and unexpectedly if
        # the version of clang that generated the .ast files is
        # different from the python clang library
        sys.stderr.write("  processing ast file {}\n".format(path))
        sys.stderr.flush()
        for cursor in tu.cursor.get_children():
            if (cursor.kind == CursorKind.FUNCTION_DECL or
                cursor.kind == CursorKind.VAR_DECL):
                decl = ci.add_decl(cursor)
                ci.walk_decl(cursor, decl)
    return ci


def main():
    parser = argparse.ArgumentParser(description='Print call trees')
    parser.add_argument('-d', '--directory', dest='directory', required=False,
                        help='directory of .ast files generated by clang')
    parser.add_argument('-o', '--output', dest='output', required=False,
                        help='output JSON file to cache call graphs')
    parser.add_argument('-i', '--input', dest='input', required=False,
                        help='input JSON file of cached call graphs')
    parser.add_argument('-f', '--function', dest='function', required=False,
                        help='function name to generate call graph for')
    parser.add_argument('--depth', dest='depth', required=False,
                        type=int, default=None,
                        help='how many calls deep to print')
    parser.add_argument('-p', '--prune', dest='prune', required=False,
                        action='store_true',
                        help='prune duplicate branches of tree')
    parser.add_argument('-c', '--callers', dest='callers', required=False,
                        action='store_true',
                        help='print callers instead of calls')
    parser.add_argument('-l', '--leaves', dest='leaves', required=False,
                        action='store_true',
                        help='only print leaf functions')
    args = parser.parse_args()
    _default_cache_filename = "print_callers.json"

    print("Printing function {}{}{}".format(
            "callers" if args.callers else "calls",
            ", prune duplicates (marked with **)" if args.prune else "",
            ", only leaf nodes" if args.leaves else ""))

    if args.directory:
        ci = _ast_files_to_callinfo(args.directory)
    elif args.input:
        ci = CallInfo.load(args.input)
    else:
        try:
            ci = CallInfo.load(_default_cache_filename)
        except Exception:
            print("Need either a previously generated cache file to input or a"
                  " directory of .ast files from clang")
            sys.exit(1)

    if args.directory:
        if args.output:
            ci.dump(args.output)
        else:
            ci.dump(_default_cache_filename)

    if args.function:
        decl = ci.decls.get(args.function)
        if decl:
            if args.callers:
                decl.print_callers(args)
            else:
                decl.print_calls(args)
        else:
            print("function {} not found".format(args.function))
    else:
        print("no function specified to print call tree")

if __name__ == "__main__":
    main()
