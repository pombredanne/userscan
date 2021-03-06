:title: fc-userscan
:title_upper: FC-USERSCAN
:subtitle: Scans directories and registers for Nix store references
:manual_section: 1
:copyright: Flying Circus Internet Operations GmbH and contributors
:author: Christian Kauhaus
:version: @version@

SYNOPSIS
========

**fc-userscan** [*OPTIONS*] *STARTDIR*


DESCRIPTION
===========

Scans directories recursively for files containing references to the Nix store
(i.e., manually compiled programs). Found references are registered as Nix
garbage collector roots to protect referenced derivations from garbage
collection.


OPTIONS
=======

**--cache**, **-c** *FILE*
    Preserves scan results between runs to avoid re-scanning unchanged files.
    For each file, the ctime inode attribute is used to decide whether it has
    been changed or not.

**--color**, **-C** [ **always** | **never** | **auto** ]
    Turns on funky colorful output. If set to **auto**, color is on only if run
    in a terminal.

**--debug**, **-d**
    Shows every file opened and lots of other stuff. Implies **--verbose**.

**--exclude**, **-e** *GLOB*
    Don't scan files matching *GLOB*. Note that matching directories are
    completely left out so that contained files skipped even when they are
    matched by a subsequent **--include** option. Exclude globs should be given
    in gitignore(5) format. This option may be given multiple times.

**--exclude-from**, **-E** *FILE*
    Like **--exclude**, but reads globs from *FILE*. Note that globs can be
    inverted (i.e., explicit include) by prefixing them with an exclamation mark
    (!). The format is further described in gitignore(5).

**--help**, **-h**
    Prints verbose or brief options overview.

**--include**, **-i**
    Scans only files matching *GLOB*. Handy for restricting scans to a few types
    of files or to override broader exclude globs given before.

**--cache-limit**, **-L**
    Limits cache capacity to N inodes. fc-userscan will abort when trying to
    save more than N entries in the cache. This will effectively cap memory and
    disk usage.

**--list**, **-l**
    Only prints found store references while scanning, but does not register
    them. Can be used in conjunctions with **--register**.

**--oneline**, **-1**
    When in list mode, each file is printed together with its references on the
    same line. Automatic post-processing may be easier using this format.
    If not given, the file and its references are printed on separate line.

**--quickcheck**, **-q** *SIZE*
    Improves performance for large files: if no single Nix store reference is
    present in the first *SIZE* kilobytes of a file, the rest of the file is
    skipped. There are usually store references present somewhere near the start
    of a file if a file contains references at all.

**--register**, **-r**
    Registers GC roots for newly found references and removes GC roots for
    references that are no longer valid. This is the default unless **--list**
    is given.

**--statistics**, **--stats**, **-S**
    Prints scanned files and read bytes per file type at the end of the run.
    This may help to fine-tune exclude lists.

**--stutter**, **-s** *SLEEPTIME*
    Pauses *SLEEPTIME* milliseconds after each opened file to lower I/O impact.

**--unzip**, **-z** *GLOB[,GLOB...]*
    Unpacks files matching GLOB as ZIP archives and scans all contained
    files. Accepts a comma-separated list of glob patterns.

**--verbose** | **-v**
    Generally increases output level. Prints newly created/deleted GC roots,
    processed totals, and cache hit rate for example.

**--version** | **-V**
    Prints program version and exits.


EXIT STATUS
===========

**2** if the program has been terminated due to hard errors like filures to
create GC store references or problems while reading a cache file.

**1** if there were less critical problems like I/O errors while reading
individual files or insufficient permissions.

**0** if no problems were encountered.


FILES
=====

~/.userscan-ignore
    Additional exclude/include globs are read from this file if it exists. The
    format of this file is the same as in gitignore(5).

/nix/var/nix/gcroots/profiles/per-user
    Garbage collection roots generated by fc-userscan are created in
    subdirectories of the per-user GC dir. For example, if fc-userscan is run by
    user joe, Nix store references found in **/lib/rc** are registered in
    **/nix/var/nix/gcroots/profiles/per-user/joe/lib/rc**.

/nix/store
    While scanning for Nix store references, the standard Nix store prefix is
    hard-coded for performance reasons. Alternative Nix store locations are not
    supported.


NOTES
=====

Partial reading of cache files is not supported. This means that once a cache
file has been truncated or otherwise damaged, fc-userscan will bail out while
trying to read the cache file. Delete the cache file in this case. It will be
created automatcally on the next run.


BUGS
====

Unpacking of large ZIP archives or those containing very many files is unduly
slow. Try to avoid **-z** if possible.


EXAMPLES
========

Scan files and register references starting at the current directory. Give a
high-level summary and a break-down per extension:

**fc-userscan -v -S .**

List found references, but don't register them:

**fc-userscan -l .**

Scan home dir, using a cache and an exclude file:

**fc-userscan -c ~/.cache/userscan -E /etc/userscan/exclude ~**


SEE ALSO
========

nix-collect-garbage(1), nix-store(1), gitignore(5)
