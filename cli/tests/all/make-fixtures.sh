rm -rf fixtures/
mkdir -p fixtures

printf "docdir: {docdir}\n\nf: _fixtures/a.adoc_\n\ninclude::b.adoc[]\n" > fixtures/a.adoc
printf "docdir: {docdir}\n\nf: _fixtures/b.adoc_\n" > fixtures/b.adoc

printf "f: _fixtures/attrs.adoc_\n\n"       >> fixtures/attrs.adoc
printf "docdir: {docdir}\n\n"               >> fixtures/attrs.adoc
printf "docfile: {docfile}\n\n"             >> fixtures/attrs.adoc
printf "docfilesuffix: {docfilesuffix}\n\n" >> fixtures/attrs.adoc
printf "docname: {docname}\n\n"             >> fixtures/attrs.adoc
