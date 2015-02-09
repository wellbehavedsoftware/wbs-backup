#[derive (RustcEncodable, RustcDecodable)]
pub struct DiskJobConfig {

	pub name: String,

	pub sync_script: Option <String>,
	pub sync_log: Option <String>,

	pub snapshot_script: Option <String>,
	pub snapshot_log: Option <String>,

}

#[derive (RustcEncodable, RustcDecodable)]
pub struct DiskConfig {

	pub state: String,
	pub lock: String,

	pub jobs: Vec <DiskJobConfig>,

}
