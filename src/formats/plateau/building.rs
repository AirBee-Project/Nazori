use kasane_logic::Coordinate;
use ordered_float::OrderedFloat;
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::io::{BufRead, Cursor};

/// 解析中のタグ状態
#[derive(Debug, Clone, Copy, PartialEq)]
enum TargetTag {
    None,
    UroBuildingId,
    UroCity,
    BldgClass,
    MeasuredHeight,
    Lod1HeightType,
    UroPrefecture,
    BldgUsage,
    PosList,
}

/// LOD 状態
#[derive(Debug, Clone, Copy, PartialEq)]
enum LodLevel {
    None,
    Lod1,
    Lod2,
}

/// local-name 判定（prefix 揺れ対策）
fn is_local(name: &[u8], local: &[u8]) -> bool {
    name == local || name.ends_with(local)
}

/// Building を (Attribute, Shape) のペアで返すパーサ
pub(crate) struct BuildingParser<R: BufRead> {
    reader: Reader<R>,
    buf: Vec<u8>,

    current_attribute: BuildingAttribute,
    current_tag: TargetTag,
    current_lod: LodLevel,
    lod1_surfaces: Vec<Vec<Coordinate>>,
    lod2_surfaces: Vec<Vec<Coordinate>>,
    current_ring: Vec<Coordinate>,
    pos_text_buf: String,
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub(crate) struct BuildingAttribute {
    pub gml_id: String,
    pub uro_building_id: String,
    pub uro_city_code: String,
    pub class_code: String,

    pub measured_height: Option<OrderedFloat<f64>>,
    pub lod1_height_type: Option<i32>,
    pub uro_prefecture_code: Option<String>,
    pub usage_code: Option<i32>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct BuildingShape {
    pub surfaces: Vec<Vec<Coordinate>>,
}

impl BuildingParser<Cursor<Vec<u8>>> {
    pub fn from_bytes(input: &[u8]) -> Self {
        Self::new(Cursor::new(input.to_vec()))
    }
}

impl<R: BufRead> BuildingParser<R> {
    pub fn new(reader: R) -> Self {
        let mut reader = Reader::from_reader(reader);
        reader.config_mut().trim_text(true);

        Self {
            reader,
            buf: Vec::with_capacity(8192),

            current_attribute: BuildingAttribute::default(),
            current_tag: TargetTag::None,
            current_lod: LodLevel::None,

            lod1_surfaces: Vec::new(),
            lod2_surfaces: Vec::new(),

            current_ring: Vec::new(),
            pos_text_buf: String::new(),
        }
    }

    /// bldg:Building 開始タグから gml:id を抽出
    fn parse_building_attributes(attr: &mut BuildingAttribute, e: &BytesStart) {
        for a in e.attributes().flatten() {
            if a.key.as_ref().ends_with(b"id") {
                if let Ok(val) = a.unescape_value() {
                    attr.gml_id = val.into_owned();
                }
            }
        }
    }

    fn flush_current_polygon(&mut self) {
        if self.pos_text_buf.is_empty() {
            return;
        }

        let nums: Vec<f64> = self
            .pos_text_buf
            .split_whitespace()
            .filter_map(|v| v.parse().ok())
            .collect();

        self.current_ring.clear();
        for c in nums.chunks_exact(3) {
            if let Ok(coord) = Coordinate::new(c[0], c[1], c[2]) {
                self.current_ring.push(coord);
            }
        }

        if self.current_ring.is_empty() {
            return;
        }

        match self.current_lod {
            LodLevel::Lod1 => {
                self.lod1_surfaces
                    .push(std::mem::take(&mut self.current_ring));
            }
            LodLevel::Lod2 => {
                self.lod2_surfaces
                    .push(std::mem::take(&mut self.current_ring));
            }
            _ => {}
        }
    }
}

pub(crate) fn parse_building_shapes(xml: &[u8]) -> Vec<BuildingShape> {
    BuildingParser::from_bytes(xml)
        .map(|(_, shape)| shape)
        .collect()
}

impl<R: BufRead> Iterator for BuildingParser<R> {
    type Item = (BuildingAttribute, BuildingShape);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    n if is_local(n, b"Building") => {
                        self.current_attribute = BuildingAttribute::default();
                        Self::parse_building_attributes(&mut self.current_attribute, &e);

                        self.lod1_surfaces.clear();
                        self.lod2_surfaces.clear();

                        self.current_lod = LodLevel::None;
                        self.current_tag = TargetTag::None;
                    }

                    n if is_local(n, b"lod1Solid") => {
                        self.current_lod = LodLevel::Lod1;
                    }

                    n if is_local(n, b"lod2Solid") || is_local(n, b"lod2MultiSurface") => {
                        self.current_lod = LodLevel::Lod2;
                    }

                    n if is_local(n, b"buildingID") => {
                        self.current_tag = TargetTag::UroBuildingId;
                    }

                    n if is_local(n, b"city") => {
                        self.current_tag = TargetTag::UroCity;
                    }

                    n if is_local(n, b"class") => {
                        self.current_tag = TargetTag::BldgClass;
                    }

                    n if is_local(n, b"measuredHeight") => {
                        self.current_tag = TargetTag::MeasuredHeight;
                    }

                    n if is_local(n, b"lod1HeightType") => {
                        self.current_tag = TargetTag::Lod1HeightType;
                    }

                    n if is_local(n, b"prefecture") => {
                        self.current_tag = TargetTag::UroPrefecture;
                    }

                    n if is_local(n, b"usage") => {
                        self.current_tag = TargetTag::BldgUsage;
                    }

                    n if is_local(n, b"posList") => {
                        self.current_tag = TargetTag::PosList;
                        self.pos_text_buf.clear();
                    }

                    _ => {}
                },

                Ok(Event::Text(e)) => {
                    if let Ok(text) = e.decode() {
                        let s = text.as_ref();
                        match self.current_tag {
                            TargetTag::UroBuildingId => {
                                self.current_attribute.uro_building_id = s.to_string();
                            }
                            TargetTag::UroCity => {
                                self.current_attribute.uro_city_code = s.to_string();
                            }
                            TargetTag::BldgClass => {
                                self.current_attribute.class_code = s.to_string();
                            }
                            TargetTag::MeasuredHeight => {
                                self.current_attribute.measured_height = s.parse().ok();
                            }
                            TargetTag::Lod1HeightType => {
                                self.current_attribute.lod1_height_type = s.parse().ok();
                            }
                            TargetTag::UroPrefecture => {
                                self.current_attribute.uro_prefecture_code = Some(s.to_string());
                            }
                            TargetTag::BldgUsage => {
                                self.current_attribute.usage_code = s.parse().ok();
                            }
                            TargetTag::PosList => {
                                self.pos_text_buf.push_str(s);
                                self.pos_text_buf.push(' ');
                            }
                            TargetTag::None => {}
                        }
                    }
                }

                Ok(Event::End(e)) => match e.name().as_ref() {
                    n if is_local(n, b"Polygon") => {
                        self.flush_current_polygon();
                        self.pos_text_buf.clear();
                        self.current_tag = TargetTag::None;
                    }

                    n if is_local(n, b"lod1Solid")
                        || is_local(n, b"lod2Solid")
                        || is_local(n, b"lod2MultiSurface") =>
                    {
                        self.current_lod = LodLevel::None;
                    }

                    n if is_local(n, b"Building") => {
                        let surfaces = if !self.lod2_surfaces.is_empty() {
                            std::mem::take(&mut self.lod2_surfaces)
                        } else {
                            std::mem::take(&mut self.lod1_surfaces)
                        };
                        let shape = BuildingShape { surfaces };
                        let attribute = std::mem::take(&mut self.current_attribute);
                        return Some((attribute, shape));
                    }

                    _ => {
                        self.current_tag = TargetTag::None;
                    }
                },

                Ok(Event::Eof) => break,
                Err(e) => {
                    eprintln!("XML parse error: {e}");
                    break;
                }
                _ => {}
            }

            self.buf.clear();
        }

        None
    }
}
