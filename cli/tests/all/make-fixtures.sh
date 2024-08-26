rm -rf fixtures/
mkdir -p fixtures

printf "docdir: {docdir}\n\nf: _fixtures/a.adoc_\n\ninclude::b.adoc[]\n" > fixtures/a.adoc
printf "docdir: {docdir}\n\nf: _fixtures/b.adoc_\n" > fixtures/b.adoc
