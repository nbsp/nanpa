NANPA-CHANGESET(5)

# NAME

nanpa-changeset - configuration format for changeset files under _.nanpa/_

# SYNTAX

Files should be placed under the _.nanpa/_ directory at package root, and must
end with _.kdl_.

Changeset files are KDL configuration files: refer to <https://kdl.dev> for
more information.

A changeset file consists of one or more nodes. Each node must be named _major_,
_minor_, or _patch_, and have the following attributes:

- A _type_ property
- An optional _package_ property 
- A single string argument containing the changeset description

Files must be encoded in UTF-8.

## Type

The *type* property specifies what kind of change this changeset describes, and
requires one of Keep a Changelog's types.

The following types are allowed:

- _added_
- _changed_
- _deprecated_
- _removed_
- _fixed_
- _security_

## Package

If this package contains subpackages, the *package* property must be defined,
and be a relative path from the project root to the subproject.

# EXAMPLE

```
minor type="added" "Add thing to stuff"
patch type="changed" "Change foo to bar"
```

# SEE ALSO

_nanpa_(1), _nanparc_(5)
