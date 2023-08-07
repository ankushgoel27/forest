use super::*;
use fvm_ipld_encoding::RawBytes;

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct RawBytesLotusJson(#[serde(with = "base64_standard")] Vec<u8>);

impl HasLotusJson for RawBytes {
    type LotusJson = RawBytesLotusJson;

    fn snapshots() -> Vec<(serde_json::Value, Self)> {
        vec![(
            json!("aGVsbG8gd29ybGQh"),
            RawBytes::new(Vec::from_iter(*b"hello world!")),
        )]
    }
}

impl From<RawBytes> for RawBytesLotusJson {
    fn from(value: RawBytes) -> Self {
        RawBytesLotusJson(Vec::from(value))
    }
}

impl From<RawBytesLotusJson> for RawBytes {
    fn from(value: RawBytesLotusJson) -> Self {
        Self::from(value.0)
    }
}

#[test]
fn snapshot_raw_bytes() {
    assert_all_snapshots::<fvm_ipld_encoding::RawBytes>();
}

#[cfg(test)]
quickcheck! {
    fn quickcheck_raw_bytes(val: Vec<u8>) -> () {
        assert_via_json(RawBytes::new(val))
    }
}
