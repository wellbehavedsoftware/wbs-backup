use protobuf::ProtobufError;

pub fn protobuf_result <Type> (
	result: Result <Type, ProtobufError>,
) -> Result <Type, String> {

	result.map_err (
		|protobuf_error|
		protobuf_error.description ().to_string ()
	)

}

// ex: noet ts=4 filetype=rust
