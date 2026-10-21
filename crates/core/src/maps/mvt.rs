//! The parts of a Mapbox Vector Tile the street map draws: layer names,
//! string tags and line and polygon geometry. Malformed input ends a layer
//! or feature early; it never panics.

pub const LINE: u64 = 2;
pub const POLYGON: u64 = 3;

pub struct Layer<'a> {
    pub name: &'a str,
    pub extent: u32,
    keys: Vec<&'a str>,
    values: Vec<Option<&'a str>>,
    features: Vec<&'a [u8]>,
}

pub struct Feature {
    pub kind: u64,
    tags: Vec<u32>,
    geometry: Vec<u32>,
}

enum Wire<'a> {
    Varint(u64),
    Bytes(&'a [u8]),
    Skipped,
}

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }
    fn varint(&mut self) -> Option<u64> {
        let mut value = 0u64;
        for shift in (0..64).step_by(7) {
            let byte = *self.buf.get(self.pos)?;
            self.pos += 1;
            value |= u64::from(byte & 0x7f) << shift;
            if byte < 0x80 {
                return Some(value);
            }
        }
        None
    }
    fn skip(&mut self, n: usize) -> Option<()> {
        self.pos = self.pos.checked_add(n).filter(|end| *end <= self.buf.len())?;
        Some(())
    }
    /// The next field number and value; `None` at the end or on bad input.
    fn field(&mut self) -> Option<(u64, Wire<'a>)> {
        if self.pos >= self.buf.len() {
            return None;
        }
        let key = self.varint()?;
        let wire = match key & 7 {
            0 => Wire::Varint(self.varint()?),
            1 => {
                self.skip(8)?;
                Wire::Skipped
            }
            2 => {
                let len = usize::try_from(self.varint()?).ok()?;
                let start = self.pos;
                self.skip(len)?;
                Wire::Bytes(&self.buf[start..self.pos])
            }
            5 => {
                self.skip(4)?;
                Wire::Skipped
            }
            _ => return None,
        };
        Some((key >> 3, wire))
    }
}

fn packed(bytes: &[u8]) -> Vec<u32> {
    let mut reader = Reader::new(bytes);
    let mut out = Vec::with_capacity(bytes.len() / 2);
    while reader.pos < bytes.len() {
        match reader.varint() {
            Some(v) => out.push(v as u32),
            None => break,
        }
    }
    out
}

/// Every layer in a tile, with its features still encoded.
pub fn layers(tile: &[u8]) -> Vec<Layer<'_>> {
    let mut reader = Reader::new(tile);
    let mut out = vec![];
    while let Some((field, wire)) = reader.field() {
        if let (3, Wire::Bytes(bytes)) = (field, wire) {
            if let Some(layer) = layer(bytes) {
                out.push(layer);
            }
        }
    }
    out
}

fn layer(bytes: &[u8]) -> Option<Layer<'_>> {
    let mut reader = Reader::new(bytes);
    let mut layer = Layer { name: "", extent: 4096, keys: vec![], values: vec![], features: vec![] };
    while let Some((field, wire)) = reader.field() {
        match (field, wire) {
            (1, Wire::Bytes(b)) => layer.name = std::str::from_utf8(b).ok()?,
            (2, Wire::Bytes(b)) => layer.features.push(b),
            (3, Wire::Bytes(b)) => layer.keys.push(std::str::from_utf8(b).unwrap_or("")),
            (4, Wire::Bytes(b)) => layer.values.push(string_value(b)),
            (5, Wire::Varint(v)) => layer.extent = u32::try_from(v).ok().filter(|e| *e > 0)?,
            _ => {}
        }
    }
    Some(layer)
}

fn string_value(bytes: &[u8]) -> Option<&str> {
    let mut reader = Reader::new(bytes);
    while let Some((field, wire)) = reader.field() {
        if let (1, Wire::Bytes(b)) = (field, wire) {
            return std::str::from_utf8(b).ok();
        }
    }
    None
}

impl Layer<'_> {
    pub fn features(&self) -> impl Iterator<Item = Feature> + '_ {
        self.features.iter().map(|bytes| {
            let mut reader = Reader::new(bytes);
            let mut feature = Feature { kind: 0, tags: vec![], geometry: vec![] };
            while let Some((field, wire)) = reader.field() {
                match (field, wire) {
                    (2, Wire::Bytes(b)) => feature.tags = packed(b),
                    (3, Wire::Varint(v)) => feature.kind = v,
                    (4, Wire::Bytes(b)) => feature.geometry = packed(b),
                    _ => {}
                }
            }
            feature
        })
    }
    /// A feature's string value for `key`, if it has one.
    pub fn tag(&self, feature: &Feature, key: &str) -> Option<&str> {
        feature.tags.as_chunks::<2>().0.iter().find_map(|&[k, v]| {
            (self.keys.get(k as usize) == Some(&key))
                .then(|| self.values.get(v as usize).copied().flatten())
                .flatten()
        })
    }
}

fn zigzag(n: u32) -> i64 {
    i64::from(n >> 1) ^ -i64::from(n & 1)
}

impl Feature {
    /// Lines, or polygon rings closed by repeating their first point, in tile
    /// units (0 to the layer's extent, y down).
    pub fn paths(&self) -> Vec<Vec<(i64, i64)>> {
        let g = &self.geometry;
        let (mut paths, mut current) = (vec![], vec![]);
        let (mut x, mut y, mut i) = (0i64, 0i64, 0usize);
        'commands: while i < g.len() {
            let (command, count) = (g[i] & 7, g[i] >> 3);
            i += 1;
            match command {
                1 | 2 => {
                    for _ in 0..count {
                        let (Some(&dx), Some(&dy)) = (g.get(i), g.get(i + 1)) else { break 'commands };
                        i += 2;
                        x += zigzag(dx);
                        y += zigzag(dy);
                        if command == 1 && !current.is_empty() {
                            paths.push(std::mem::take(&mut current));
                        }
                        current.push((x, y));
                    }
                }
                7 => {
                    if let Some(&first) = current.first() {
                        current.push(first);
                        paths.push(std::mem::take(&mut current));
                    }
                }
                _ => break,
            }
        }
        if !current.is_empty() {
            paths.push(current);
        }
        paths
    }
}
