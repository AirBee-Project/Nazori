use kasane_logic::Error as SpatialError;
use quick_xml::Error as XmlError;

/// ライブラリ全体の上位エラー。
#[derive(Debug)]
pub enum Error {
    Xml(XmlError),
    Kasane(SpatialError),
    /// 入力データが途中で切れている、または必要な情報が足りない。
    IncompleteInput,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Xml(err) => write!(f, "XML parse error: {err}"),
            Self::Kasane(err) => write!(f, "spatial conversion error: {err}"),
            Self::IncompleteInput => write!(f, "input data is incomplete"),
        }
    }
}

impl std::error::Error for Error {}

impl From<XmlError> for Error {
    fn from(value: XmlError) -> Self {
        Self::Xml(value)
    }
}

impl From<SpatialError> for Error {
    fn from(value: SpatialError) -> Self {
        Self::Kasane(value)
    }
}
