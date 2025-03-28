# NB: we generate these with bash commands for easily replicating tests
# inside of an asciidoctor docker container, to verify compatibility

rm -rf fixtures/gen
mkdir -p fixtures/gen/sub

printf "docdir: {docdir}\n\n"               >> fixtures/gen/a.adoc
printf "f: _fixtures/gen/a.adoc_\n\n"       >> fixtures/gen/a.adoc
printf "include::b.adoc[]\n"                >> fixtures/gen/a.adoc
printf "docdir: {docdir}\n\n"               >> fixtures/gen/b.adoc
printf "f: _fixtures/gen/b.adoc_\n"         >> fixtures/gen/b.adoc

printf "f: _fixtures/gen/attrs.adoc_\n\n"   >> fixtures/gen/attrs.adoc
printf "docdir: {docdir}\n\n"               >> fixtures/gen/attrs.adoc
printf "docfile: {docfile}\n\n"             >> fixtures/gen/attrs.adoc
printf "docfilesuffix: {docfilesuffix}\n\n" >> fixtures/gen/attrs.adoc
printf "docname: {docname}\n\n"             >> fixtures/gen/attrs.adoc

# replicates a test in asciidoctor
printf "first line of parent\n\n"           >> fixtures/gen/parent-include.adoc
printf "include::child-include.adoc[]\n\n"  >> fixtures/gen/parent-include.adoc
printf "last line of parent\n"              >> fixtures/gen/parent-include.adoc
printf "first line of child\n\n"            >> fixtures/gen/child-include.adoc
printf "include::gchild-include.adoc[]\n\n" >> fixtures/gen/child-include.adoc
printf "last line of child\n"               >> fixtures/gen/child-include.adoc
printf "first line of grandchild\n\n"       >> fixtures/gen/gchild-include.adoc
printf "last line of grandchild\n"          >> fixtures/gen/gchild-include.adoc

# replicates a test in asciidoctor
printf "first line of outer\n\n"           >> fixtures/gen/relative-include.adoc
printf "include::sub/middle.adoc[]\n\n"    >> fixtures/gen/relative-include.adoc
printf "last line of outer\n"              >> fixtures/gen/relative-include.adoc
printf "first line of middle\n\n"          >> fixtures/gen/sub/middle.adoc
printf "include::inner.adoc[]\n\n"         >> fixtures/gen/sub/middle.adoc
printf "last line of middle\n"             >> fixtures/gen/sub/middle.adoc
printf "only line of inner\n"              >> fixtures/gen/sub/inner.adoc

printf "with trailing space    \n"          >> fixtures/gen/trailing.adoc
printf -- "----\ninclude::trailing.adoc[]\n">> fixtures/gen/preproc.adoc
printf -- "----\n\n"                        >> fixtures/gen/preproc.adoc

# case insensitivity
printf "include::sub/inNER.adoc[]\n\n"     >> fixtures/gen/case-fail.adoc

