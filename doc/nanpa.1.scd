NANPA(1)

# NAME

nanpa - a language-agnostic release manager

# SYNOPSIS

*nanpa* help

*nanpa* [-hv]

*nanpa* _command_ [_arguments_...]

# DESCRIPTION

*nanpa help* (or *nanpa --help* or *nanpa -h*) prints help text.

*nanpa version* prints the current package's version, or its subpackages'
versions, if its config has a _packages_ option.

*nanpa changesets* [_package_] reads changeset files in _.nanpa/\*.kdl_ into a
_CHANGELOG.md_ file, formatted in the style of _Keep a Changelog_, then bumps
the package version accordingly. If _package_ isn't provided, *nanpa* will
traverse up the tree until it finds a *.nanparc* file. If the found *.nanparc*
file includes subpackages, *nanpa changesets* will be run for each of them.
If called with the *--pre* argument, the new version will be calculated as a
prerelease of the desired type.

*nanpa add* <major|minor|patch> adds a changeset file to be handled by *nanpa
changesets*. you may feed it the type, subpackage and description at the command
line, and it will open an editor with them pre-written.

*nanpa bump* <major|minor|patch|pre _prerelease_|_custom_> [_package_] manually
bumps the package version, either according to Semantic Versioning or with a
custom version. If _package_ isn't provided, *nanpa* will traverse up the tree
until it finds a *.nanparc* file.

*nanpa list-languages* lists the supported languages that *nanpa* can bump
automatically.

# SEE ALSO

_nanparc_(5), _nanpa-changeset_(5)
