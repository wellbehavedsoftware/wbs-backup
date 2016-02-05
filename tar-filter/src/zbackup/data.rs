use zbackup::proto;

pub type IndexEntry = (
	proto::IndexBundleHeader,
	proto::BundleInfo,
);
