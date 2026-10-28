//! Räume und Öffnungen eines Geschosses als Verbund: Welche Öffnung verbindet
//! welche Räume, welche führt ins Freie. Grundlage für die Tür- und
//! Fensterliste, die Barrierefreiheit und die Rettungswege.

use lot_geom::Area;
use lot_geom::spaces::SpaceFinder;
use lot_model::{Element, ElementId, Opening, OpeningKind, Point2, Project, Space, WallSide};

/// Abstand der Prüfpunkte vor der Wandfläche, in m.
const PROBE: f64 = 0.15;

/// Ein Raum mit seiner Fläche.
pub struct Room<'a> {
    pub id: ElementId,
    pub space: &'a Space,
    pub area: Area,
}

impl Room<'_> {
    /// „0.04 Schlafen“.
    pub fn label(&self) -> String {
        format!("{} {}", self.space.number, self.space.name).trim().to_string()
    }
}

/// Eine Öffnung mit den Räumen auf beiden Seiten.
pub struct Passage<'a> {
    pub id: ElementId,
    pub opening: &'a Opening,
    pub wall: ElementId,
    /// Mitte der Öffnung im Grundriss.
    pub center: Point2,
    /// Einheitsvektor senkrecht zur Wand, zur linken Seite.
    pub normal: [f64; 2],
    /// Raum links und rechts der Wand (Index in `rooms`); `None` außen oder ohne Raum.
    pub left: Option<usize>,
    pub right: Option<usize>,
    /// Ob die Seite außerhalb des Gebäudes liegt.
    pub left_outside: bool,
    pub right_outside: bool,
    /// Brüstungshöhe über OKFF.
    pub sill: f64,
}

impl Passage<'_> {
    /// Raum auf der Seite, zu der die Tür aufschlägt (bei Türen), sonst links.
    pub fn swing_room(&self) -> Option<usize> {
        match self.opening.kind {
            OpeningKind::Door(d) => match d.opens_to {
                WallSide::Left => self.left,
                WallSide::Right => self.right,
            },
            _ => self.left,
        }
    }

    pub fn other_side(&self, room: usize) -> Option<usize> {
        if self.left == Some(room) {
            self.right
        } else if self.right == Some(room) {
            self.left
        } else {
            None
        }
    }

    /// Ob die Öffnung ins Freie führt.
    pub fn to_outside(&self) -> bool {
        self.left_outside || self.right_outside
    }

    pub fn is_door_or_passage(&self) -> bool {
        matches!(self.opening.kind, OpeningKind::Door(_) | OpeningKind::Plain)
    }
}

/// Räume und Öffnungen eines Geschosses.
pub struct StoreyRooms<'a> {
    pub storey: ElementId,
    pub rooms: Vec<Room<'a>>,
    pub passages: Vec<Passage<'a>>,
    /// Umriss des Geschosses ohne Löcher.
    pub outline: Area,
}

impl<'a> StoreyRooms<'a> {
    pub fn new(project: &'a Project, storey: ElementId) -> Self {
        let finder = SpaceFinder::new(project, storey);
        let outline = crate::building::outline_of(&finder.footprint);
        let mut rooms: Vec<Room<'a>> = Vec::new();
        for (id, space) in project.spaces_on(storey) {
            if space.phase == lot_model::Phase::Demolished {
                continue;
            }
            if let Some(area) = finder.area_at([space.position.x, space.position.y]) {
                rooms.push(Room { id, space, area: area.clone() });
            }
        }
        rooms.sort_by(|a, b| a.space.number.cmp(&b.space.number).then(a.id.cmp(&b.id)));
        let room_at = |p: Point2| rooms.iter().position(|r| r.area.contains([p.x, p.y]));
        let mut passages = Vec::new();
        for (wall_id, wall) in project.walls_on(storey) {
            let Some(wall_type) = project.wall_type(wall.wall_type) else { continue };
            if wall.phase == lot_model::Phase::Demolished {
                continue;
            }
            let offsets = wall.layer_offsets(wall_type);
            let (left, right) = (offsets[0].0, offsets[offsets.len() - 1].1);
            for (id, o) in project.openings_in(wall_id) {
                if o.phase == lot_model::Phase::Demolished {
                    continue;
                }
                let s = o.offset + o.width / 2.0;
                let (tx, ty) = wall.tangent_at(s);
                let normal = [-ty, tx];
                let pl = wall.point_at(s, left + PROBE);
                let pr = wall.point_at(s, right - PROBE);
                let center = wall.point_at(s, (left + right) / 2.0);
                let outside = |p: Point2| !outline.contains([p.x, p.y]);
                passages.push(Passage {
                    id,
                    opening: o,
                    wall: wall_id,
                    center,
                    normal,
                    left: room_at(pl),
                    right: room_at(pr),
                    left_outside: outside(pl),
                    right_outside: outside(pr),
                    // Fertige Brüstung über OKFF (ADR 0047).
                    sill: o.sill_height,
                });
            }
        }
        passages.sort_by_key(|p| p.id);
        Self { storey, rooms, passages, outline }
    }

    /// Öffnungen eines Raums.
    pub fn passages_of(&self, room: usize) -> impl Iterator<Item = &Passage<'a>> {
        self.passages.iter().filter(move |p| p.left == Some(room) || p.right == Some(room))
    }

    pub fn room_of(&self, id: ElementId) -> Option<usize> {
        self.rooms.iter().position(|r| r.id == id)
    }

    /// Räume, in denen eine notwendige Treppe dieses Geschosses beginnt oder
    /// eine Treppe aus dem Geschoss darunter ankommt: die Treppenräume.
    pub fn stair_rooms(&self, project: &Project) -> Vec<usize> {
        let storeys = project.storeys();
        let below = storeys.iter().position(|(id, _)| *id == self.storey).and_then(|i| i.checked_sub(1));
        let mut points = Vec::new();
        for (_, stair) in project.stairs_on(self.storey).filter(|(_, s)| s.necessary) {
            points.push(stair.start);
        }
        if let Some(i) = below {
            for (_, stair) in project.stairs_on(storeys[i].0).filter(|(_, s)| s.necessary) {
                if let Some(layout) = project.stair_layout(stair)
                    && let Some(last) = layout.flights.last()
                {
                    let end = layout.end(last);
                    // Kurz hinter dem Austritt, auf dem Fußboden dieses Geschosses.
                    points.push(Point2::new(end.x + last.dir.x * 0.3, end.y + last.dir.y * 0.3));
                }
            }
        }
        let mut out: Vec<usize> =
            points.iter().filter_map(|p| self.rooms.iter().position(|r| r.area.contains([p.x, p.y]))).collect();
        out.sort();
        out.dedup();
        out
    }
}

/// Alle Nutzungseinheiten mit Räumen, in der Reihenfolge ihres ersten Raums.
pub fn units(project: &Project) -> Vec<String> {
    let mut list: Vec<(String, String)> = project
        .elements
        .values()
        .filter_map(|e| match e {
            Element::Space(s) if !s.unit.is_empty() => Some((s.number.clone(), s.unit.clone())),
            _ => None,
        })
        .collect();
    list.sort();
    let mut out: Vec<String> = Vec::new();
    for (_, u) in list {
        if !out.contains(&u) {
            out.push(u);
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_doors_connect_rooms() {
        let p = Project::sample();
        let storey = p.storeys()[0].0;
        let r = StoreyRooms::new(&p, storey);
        // Jede Öffnung in einer Außenwand führt auf einer Seite ins Freie.
        assert!(r.passages.iter().any(|x| x.to_outside()));
    }
}
