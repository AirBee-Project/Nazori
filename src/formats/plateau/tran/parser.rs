use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::io::{BufRead, Cursor};

use kasane_logic::Coordinate;

use super::model::{TranAttribute, TranShape};

#[derive(Debug, Clone, Copy, PartialEq)]
enum TargetTag {
    None,
    PosList,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LodLevel {
    None,
    Lod1,
    Lod2,
}

fn is_local(name: &[u8], local: &[u8]) -> bool {
    if name == local {
        return true;
    }
    if let Some(pos) = name.iter().position(|&b| b == b':') {
        return &name[pos + 1..] == local;
    }
    false
}

pub(crate) struct TranParser<R: BufRead> {
    reader: Reader<R>,
    buf: Vec<u8>,
    current_attribute: TranAttribute,
    current_tag: TargetTag,
    current_lod: LodLevel,
    lod1_surfaces: Vec<Vec<Coordinate>>,
    lod2_surfaces: Vec<Vec<Coordinate>>,
    current_ring: Vec<Coordinate>,
    pos_text_buf: String,
}

impl TranParser<Cursor<Vec<u8>>> {
    #[allow(unused)]
    pub fn from_bytes(input: &[u8]) -> Self {
        Self::new(Cursor::new(input.to_vec()))
    }
}

impl<R: BufRead> TranParser<R> {
    pub fn new(reader: R) -> Self {
        let mut reader = Reader::from_reader(reader);
        reader.config_mut().trim_text(true);

        Self {
            reader,
            buf: Vec::with_capacity(8192),
            current_attribute: TranAttribute::default(),
            current_tag: TargetTag::None,
            current_lod: LodLevel::None,
            lod1_surfaces: Vec::new(),
            lod2_surfaces: Vec::new(),
            current_ring: Vec::new(),
            pos_text_buf: String::new(),
        }
    }

    fn parse_id(attr: &mut TranAttribute, e: &BytesStart) {
        for a in e.attributes().flatten() {
            if a.key.as_ref().ends_with(b"id")
                && let Ok(val) = a.unescape_value()
            {
                attr.gml_id = val.into_owned();
            }
        }
    }

    fn push_pos_list(&mut self) {
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
            LodLevel::Lod1 => self
                .lod1_surfaces
                .push(std::mem::take(&mut self.current_ring)),
            LodLevel::Lod2 => self
                .lod2_surfaces
                .push(std::mem::take(&mut self.current_ring)),
            LodLevel::None => {}
        }
    }

    fn handle_start(&mut self, e: BytesStart) {
        match e.name().as_ref() {
            n if is_local(n, b"Road")
                || is_local(n, b"Railway")
                || is_local(n, b"Square")
                || is_local(n, b"Track") =>
            {
                self.current_attribute = TranAttribute::default();
                Self::parse_id(&mut self.current_attribute, &e);
                self.lod1_surfaces.clear();
                self.lod2_surfaces.clear();
                self.current_lod = LodLevel::None;
                self.current_tag = TargetTag::None;
            }
            n if is_local(n, b"lod1MultiSurface") || is_local(n, b"lod1Solid") => {
                self.current_lod = LodLevel::Lod1
            }
            n if is_local(n, b"lod2MultiSurface") || is_local(n, b"lod2Solid") => {
                self.current_lod = LodLevel::Lod2
            }
            n if is_local(n, b"posList") => {
                self.current_tag = TargetTag::PosList;
                self.pos_text_buf.clear();
            }
            _ => {}
        }
    }

    fn handle_text(&mut self, e: quick_xml::events::BytesText<'_>) {
        if let Ok(text) = e.decode() {
            let s = text.as_ref();
            match self.current_tag {
                TargetTag::PosList => {
                    self.pos_text_buf.push_str(s);
                    self.pos_text_buf.push(' ');
                }
                TargetTag::None => {}
            }
        }
    }

    fn handle_end(
        &mut self,
        e: quick_xml::events::BytesEnd<'_>,
    ) -> Option<(TranAttribute, TranShape)> {
        match e.name().as_ref() {
            n if is_local(n, b"Polygon") => {
                self.push_pos_list();
                self.pos_text_buf.clear();
                self.current_tag = TargetTag::None;
            }
            n if is_local(n, b"Road")
                || is_local(n, b"Railway")
                || is_local(n, b"Square")
                || is_local(n, b"Track") =>
            {
                let surfaces = if !self.lod2_surfaces.is_empty() {
                    std::mem::take(&mut self.lod2_surfaces)
                } else {
                    std::mem::take(&mut self.lod1_surfaces)
                };
                let shape = TranShape { surfaces };
                let attribute = std::mem::take(&mut self.current_attribute);
                return Some((attribute, shape));
            }
            n if is_local(n, b"lod1MultiSurface")
                || is_local(n, b"lod1Solid")
                || is_local(n, b"lod2MultiSurface")
                || is_local(n, b"lod2Solid") =>
            {
                self.current_lod = LodLevel::None;
            }
            _ => self.current_tag = TargetTag::None,
        }

        None
    }
}

impl<R: BufRead> Iterator for TranParser<R> {
    type Item = Result<(TranAttribute, TranShape), quick_xml::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let event = self
                .reader
                .read_event_into(&mut self.buf)
                .map(|event| event.into_owned());

            match event {
                Ok(Event::Start(e)) => self.handle_start(e.into_owned()),
                Ok(Event::Text(e)) => self.handle_text(e.into_owned()),
                Ok(Event::End(e)) => {
                    if let Some(item) = self.handle_end(e.into_owned()) {
                        return Some(Ok(item));
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Some(Err(e)),
                _ => {}
            }

            self.buf.clear();
        }

        None
    }
}
