use super::NUM_SUBDIVISIONS;

#[derive(Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum NodeIndex {
    #[default]
    ThisNode,
    Child(u8),
}

impl ToString for NodeIndex {
    fn to_string(&self) -> String {
        match self {
            Self::ThisNode => "".into(),
            Self::Child(num) => num.to_string(),
        }
    }
}

impl From<u8> for NodeIndex {
    fn from(val: u8) -> Self {
        if val == NUM_SUBDIVISIONS as u8 {
            Self::ThisNode
        } else {
            Self::Child(val)
        }
    }
}

impl From<NodeIndex> for u8 {
    fn from(val: NodeIndex) -> Self {
        match val {
            NodeIndex::ThisNode => NUM_SUBDIVISIONS as u8,
            NodeIndex::Child(num) => num,
        }
    }
}
