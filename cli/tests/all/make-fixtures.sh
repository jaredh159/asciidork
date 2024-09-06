rm -rf fixtures/
mkdir -p fixtures

printf "docdir: {docdir}\n\n"               >> fixtures/a.adoc
printf "f: _fixtures/a.adoc_\n\n"           >> fixtures/a.adoc
printf "include::b.adoc[]\n"                >> fixtures/a.adoc
printf "docdir: {docdir}\n\n"               >> fixtures/b.adoc
printf "f: _fixtures/b.adoc_\n"             >> fixtures/b.adoc

printf "f: _fixtures/attrs.adoc_\n\n"       >> fixtures/attrs.adoc
printf "docdir: {docdir}\n\n"               >> fixtures/attrs.adoc
printf "docfile: {docfile}\n\n"             >> fixtures/attrs.adoc
printf "docfilesuffix: {docfilesuffix}\n\n" >> fixtures/attrs.adoc
printf "docname: {docname}\n\n"             >> fixtures/attrs.adoc

# replicates a test in asciidoctor
printf "first line of parent\n\n"           >> fixtures/parent-include.adoc
printf "include::child-include.adoc[]\n\n"  >> fixtures/parent-include.adoc
printf "last line of parent\n"              >> fixtures/parent-include.adoc
printf "first line of child\n\n"            >> fixtures/child-include.adoc
printf "include::gchild-include.adoc[]\n\n" >> fixtures/child-include.adoc
printf "last line of child\n"               >> fixtures/child-include.adoc
printf "first line of grandchild\n\n"       >> fixtures/gchild-include.adoc
printf "last line of grandchild\n"          >> fixtures/gchild-include.adoc
