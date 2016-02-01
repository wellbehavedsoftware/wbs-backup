# Tar Filter

This is a proof-of-concept tool which reorders the contents of a tar file,
storing all of the file contents at the start, with the metadata at the end, and
finally a table of the new position of the blocks which can be used to
reconstruct the original file.

It's designed to work with [ZBackup](http://zbackup.org/), which should be able
to more efficiently deduplicate the file data, which is less likely to change
than the metadata, in many cases.

## Wish list

* Detect and store extended metadata, eg long filenames, at the end of the pack.
* Checksums.
* Buffered reads and writes.
* Try aligning all data on 64k boundaries, zbackup's default block size
