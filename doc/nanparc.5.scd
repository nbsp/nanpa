NANPARC(5)

# NAME

.nanparc - configuration format for _nanpa_(1)

# SYNTAX

The *.nanparc* file should be placed at the package root. Inside it should be a
list of option-value statements, separated by newlines:

	_option_ _value_ [_value_...]

All options, except for _packages_, take only one value. The rest are ignored.

Options must be specified only once.

A file must have either _packages_ or _version_ and _name_, but not both.

Files must be encoded in UTF-8.

## Packages

Traverse the paths to find additional *.nanparc* files and get packages from
them. Supports globbing and recursion.

Example:
	packages foo bar baz/\*

## Version

Specify package version. Can either be a Semantic Versioning version or an
custom blob.

Example:
	version 0.1.0-alpha.1

## Name

Specify package name. Currently unused.

Example:
	name foo

## Language

Bump version on supported toolchains. Run *nanpa list-languages* for a list of
supported languages.

Example:
	language cargo

## Custom

Run a file on package version bump. Can be a relative or absolute path. Does not
take arguments, and does not respect _$PATH_. An environmental variable
_$VERSION_ is supplied to the file, which contains the package version.

Example:
	custom bump_python.sh

## Comments and whitespace

Empty lines and trailing whitespace are ignored.

Lines beginning with *#* are ignored.

# SEE ALSO

_nanpa_(1), _nanpa-changeset_(5)
