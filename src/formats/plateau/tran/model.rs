use kasane_logic::Coordinate;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub(crate) struct TranAttribute {
    pub gml_id: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct TranShape {
    pub surfaces: Vec<Vec<Coordinate>>,
}
