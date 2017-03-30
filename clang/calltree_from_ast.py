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

    def add_caller(self, decl):
        self.callers[decl.name] = decl

    def print_callers(self, depth=None, abridged=False):
        print("Callers:")
        self.print_tree(lambda node: node.callers, depth, abridged)

    def print_tree(self, get_children, depth=None, abridged=False):
        l = [self.name]

        # format shamelessly cribbed from asciitree
        def recurse_print(node, lines, parent_set, indent, curr_depth):
            children = get_children(node)
            length = len(children)
            for i, (name, caller) in enumerate(children.items()):
                if name in parent_set:
                    # don't infinite loop for e.g. recursive functions
                    lines.append("{}+-- {}**".format(indent, name))
                    continue


                lines.append("{}+-- {}".format(indent, name))
                if i + 1 == length:
                    new_indent = indent + "    "
                else:
                    new_indent = indent + "|   "
                if depth is None or curr_depth < depth:
                    if abridged:
                        new_set = parent_set
                    else:
                        new_set = parent_set.copy()
                    new_set.add(name)
                    recurse_print(caller, lines, new_set, new_indent,
                                  curr_depth + 1)

        recurse_print(self, l, set(), "", 0)
        print('\n'.join(l))

    def __str__(self):
        return "{}".format(self.name)


class Calls(object):
    def __init__(self):
        self.decls = dict()
        self._globals = dict()

    def add_global(self, cursor):
        self._globals[cursor.get_usr()] = cursor

    def add_decl(self, cursor):
        name = cursor.spelling
        return self.decls.setdefault(name, Decl(name))

    def walk_decl(self, cursor, decl):
        if ((cursor.kind == CursorKind.CALL_EXPR) or
           (cursor.kind == CursorKind.DECL_REF_EXPR and
            (cursor.type.kind == TypeKind.FUNCTIONNOPROTO or
             cursor.type.kind == TypeKind.FUNCTIONPROTO))):
            call_func = self.add_decl(cursor)
            call_func.add_caller(decl)
        elif cursor.kind == CursorKind.DECL_REF_EXPR:
            # this is for if you stick a function ptr in a global variable
            # this probably doesn't catch every case
            parent = cursor.referenced.canonical
            match = self._globals.get(parent.get_usr())
            if match:
                call_func = self.add_decl(match)
                call_func.add_caller(decl)

        for child in cursor.get_children():
            self.walk_decl(child, decl)

    def dump(self, filename):
        sys.stderr.write("saving output to {}\n".format(filename))
        sys.stderr.flush()
        callers = {name: list(decl.callers.keys())
                   for name, decl in self.decls.items()}
        with open(filename, "w") as f:
            json.dump({'callers': callers}, f)

    @classmethod
    def load(cls, filename):
        sys.stderr.write("loading input from {}\n".format(filename))
        sys.stderr.flush()
        with open(filename, "r") as f:
            d = json.load(f)

        callers = d.get('callers')
        if not callers:
            raise Exception("No 'callers' object found")

        calls = Calls()
        for name, decl_callers in callers.items():
            # might have been created by a previous decl who calls this func
            decl = calls.decls.get(name, Decl(name))
            for caller_name in decl_callers:
                caller_decl = calls.decls.get(caller_name, Decl(caller_name))
                decl.add_caller(caller_decl)
                calls.decls[caller_name] = caller_decl
            calls.decls[name] = decl
        return calls


def _ast_files(d):
    for root, _, files in os.walk(d):
        for f in files:
            if not f.endswith('.ast'):
                continue
            yield os.path.join(root, f)


def _ast_files_to_calls(directory):
    index = Index(conf.lib.clang_createIndex(False, True))

    # don't list comprehend so we can get better error reporting
    units = []
    for path in _ast_files(directory):
        try:
            units.append((os.path.abspath(path),
                          TranslationUnit.from_ast_file(path, index)))
        except Exception as e:
            print("error parsing {}".format(path))
            print(e.args)
            print(e.message)
            raise

    c = Calls()

    for path, tu in units:
        for cursor in tu.cursor.get_children():
            # seems hacky, probably misses c++ cases
            # stuff from includes has the include's filename
            if ((cursor.kind == CursorKind.VAR_DECL) and
                (cursor.location.file.name == tu.spelling)):
                c.add_global(cursor)

    for path, tu in units:
        # WARNING: this will fail silently and unexpectedly if
        # the version of clang that generated the .ast files is
        # different from the python clang library
        sys.stderr.write("  processing ast file {}\n".format(path))
        sys.stderr.flush()
        for cursor in tu.cursor.get_children():
            if (cursor.kind == CursorKind.FUNCTION_DECL or
                cursor.kind == CursorKind.VAR_DECL):
                decl = c.add_decl(cursor)
                c.walk_decl(cursor, decl)
    return c


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
    parser.add_argument('--abridged', dest='abridged', required=False,
                        action='store_true',
                        help='abridge repeated parts of tree')
    args = parser.parse_args()
    _default_cache_filename = "print_callers.json"

    if args.directory:
        calls = _ast_files_to_calls(args.directory)
    elif args.input:
        calls = Calls.load(args.input)
    else:
        try:
            calls = Calls.load(_default_cache_filename)
        except Exception:
            print("Need either a previously generated cache file to input or a"
                  " directory of .ast files from clang")
        sys.exit(1)

    if args.directory:
        if args.output:
            calls.dump(args.output)
        else:
            calls.dump(_default_cache_filename)

    if args.function:
        decl = calls.decls.get(args.function)
        if decl:
            decl.print_callers(args.depth, args.abridged)
        else:
            print("function {} not found".format(args.function))
    else:
        print("no function specified to print call tree")

if __name__ == "__main__":
    main()
