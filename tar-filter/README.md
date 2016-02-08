# Tar Filter

This is a proof-of-concept tool which reorders the contents of a tar file,
storing all of the file contents at the start, with the metadata at the end, and
finally a table of the new position of the blocks which can be used to
reconstruct the original file.

It's designed to work with [ZBackup](http://zbackup.org/), which should be able
to more efficiently deduplicate the file data, which is less likely to change
than the metadata, in many cases. It is also able to align file contents on 64k
boundaries, which increases the likelyhood of ZBackup's deduplication algorithm
finding matches in many cases.

We also include an implementation of zbackup's restore algorithm, which is able
to efficiently unpack backed up data without first extracting the uncompressed
pack data to a regular file.

## Licensing

This project is copyright by James Pharaoh <james@wellbehavedsoftware.com>. It
is released to the public under the permissive MIT license. Please see the
LICENSE file, or look up the MIT license on line, for more details.

This project also includes work by Konstantin Isakov, the original author of
ZBackup. Specifically, it includes the protobuf definintions from that project,
and of course this project could not exist without the original work on ZBackup
by Konstantin and the other contributors. Further thanks to Konstantin for
graciously releasing the protobuf definitions under the MIT licence so that they
can be included directly in this project.

## Wish list

* Checksums for pack format
* Buffered reads and writes
* Parallel zbackup restore
* Encrypted zbackup restore
* Verify checksums during zbackup restore
* Predictable ordering of tar contents
* Transparently decompress and recompress
* Nested packing of nested tars
* Padding for other file types, eg databases
