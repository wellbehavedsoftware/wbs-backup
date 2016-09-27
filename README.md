# wbs-backup

https://github.com/wellbehavedsoftware/wbs-backup

James Pharaoh <james@wellbehavedsoftware.com>

This is a collection of small backup tools, written in Rust, which are used in
our production hosting environment.

## backup-daemon

This daemon is responsible for scheduling backups. It has a simple JSON
configuration file, a simple JSON state file, and runs external scripts to
peform the backup steps.

It's used internally on many systems, but a bit rough around the edges, and
probably not yet suitable for anyone else to adopt.

[More information](tree/master/backup-daemon)

## tar-filter

This tool reorders a TAR archive, placing all of the file contents contiguously
at the start, and all of the tar headers contiguously at the end. It is designed
to increase the deduplication efficiency of zbackup.

This is a proof-of-concept, not yet used in production.

[More information](tree/master/tar-filter)

## rzbackup

This includes a partial clone of zbackup, and a server which is able to more
efficiently perform repeated restore operations by keeping cached state around
in memory and in temporary files.

It is intended to evolve into a full zbackup clone, along with an advanced
client/server model and other optimisations, and ideally a better implementation
of the client/server protocol can be designed publicly and backported to
ZBackup itself. 

[More information](tree/master/rzbackup)
