use prost::bytes::Bytes;

use crate::agglayer::protocol::types::v1::{FixedBytes20, FixedBytes32, FixedBytes65};

impl From<[u8; 32]> for FixedBytes32 {
    fn from(value: [u8; 32]) -> Self {
        FixedBytes32 {
            value: Bytes::from_owner(value),
        }
    }
}

impl TryFrom<FixedBytes32> for [u8; 32] {
    type Error = anyhow::Error;

    fn try_from(value: FixedBytes32) -> Result<Self, Self::Error> {
        if value.value.len() != 32 {
            panic!("Invalid length for FixedBytes32");
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(&value.value);

        Ok(array)
    }
}

impl TryFrom<Vec<u8>> for FixedBytes32 {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() != 32 {
            Err(anyhow::anyhow!("Invalid length for FixedBytes32"))
        } else {
            Ok(FixedBytes32 {
                value: Bytes::from_owner(value),
            })
        }
    }
}

impl TryFrom<&[u8]> for FixedBytes32 {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 32 {
            Err(anyhow::anyhow!("Invalid length for FixedBytes32"))
        } else {
            Ok(FixedBytes32 {
                value: Bytes::from_owner(value.to_vec()),
            })
        }
    }
}

impl From<[u8; 20]> for FixedBytes20 {
    fn from(value: [u8; 20]) -> Self {
        FixedBytes20 {
            value: Bytes::from_owner(value),
        }
    }
}

impl TryFrom<Vec<u8>> for FixedBytes20 {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() != 20 {
            Err(anyhow::anyhow!("Invalid length for FixedBytes20"))
        } else {
            Ok(FixedBytes20 {
                value: Bytes::from_owner(value),
            })
        }
    }
}

impl TryFrom<&[u8]> for FixedBytes20 {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 20 {
            Err(anyhow::anyhow!("Invalid length for FixedBytes20"))
        } else {
            Ok(FixedBytes20 {
                value: Bytes::from_owner(value.to_vec()),
            })
        }
    }
}

impl From<[u8; 65]> for FixedBytes65 {
    fn from(value: [u8; 65]) -> Self {
        FixedBytes65 {
            value: Bytes::from_owner(value),
        }
    }
}

impl TryFrom<Vec<u8>> for FixedBytes65 {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() != 65 {
            Err(anyhow::anyhow!("Invalid length for FixedBytes65"))
        } else {
            Ok(FixedBytes65 {
                value: Bytes::from_owner(value),
            })
        }
    }
}

impl TryFrom<&[u8]> for FixedBytes65 {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 65 {
            Err(anyhow::anyhow!("Invalid length for FixedBytes65"))
        } else {
            Ok(FixedBytes65 {
                value: Bytes::from_owner(value.to_vec()),
            })
        }
    }
}
