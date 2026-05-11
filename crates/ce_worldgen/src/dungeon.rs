//! Dungeon generation using BSP (Binary Space Partitioning).
//!
//! Recursively splits a rectangle into rooms, then connects them with corridors.
//! Guarantees all rooms are reachable (connected graph).

/// A rectangular room.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Room {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Room {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn center(&self) -> (i32, i32) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    pub fn area(&self) -> i32 {
        self.width * self.height
    }

    pub fn intersects(&self, other: &Room) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

/// A corridor connecting two rooms.
#[derive(Debug, Clone, Copy)]
pub struct Corridor {
    pub from: (i32, i32),
    pub to: (i32, i32),
}

/// Tile type for the dungeon grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Wall,
    Floor,
    Corridor,
    Door,
}

/// A generated dungeon.
#[derive(Debug, Clone)]
pub struct Dungeon {
    pub width: i32,
    pub height: i32,
    pub rooms: Vec<Room>,
    pub corridors: Vec<Corridor>,
    pub tiles: Vec<Tile>,
}

impl Dungeon {
    /// Generate a dungeon with BSP.
    pub fn generate(width: i32, height: i32, seed: u64, min_room_size: i32) -> Self {
        let mut rng_state = seed;
        let mut rooms = Vec::new();

        // BSP: recursively split the space
        let mut partitions = vec![(0i32, 0i32, width, height)];
        let max_splits = 6;

        for _ in 0..max_splits {
            let mut new_partitions = Vec::new();
            for (px, py, pw, ph) in &partitions {
                let (px, py, pw, ph) = (*px, *py, *pw, *ph);
                if pw < min_room_size * 2 + 4 && ph < min_room_size * 2 + 4 {
                    new_partitions.push((px, py, pw, ph));
                    continue;
                }

                let split_h = if pw > ph {
                    true
                } else if ph > pw {
                    false
                } else {
                    lcg_next(&mut rng_state).is_multiple_of(2)
                };

                if split_h && pw >= min_room_size * 2 + 4 {
                    let split = min_room_size
                        + 2
                        + (lcg_next(&mut rng_state) % (pw - min_room_size * 2 - 3) as u64) as i32;
                    new_partitions.push((px, py, split, ph));
                    new_partitions.push((px + split, py, pw - split, ph));
                } else if !split_h && ph >= min_room_size * 2 + 4 {
                    let split = min_room_size
                        + 2
                        + (lcg_next(&mut rng_state) % (ph - min_room_size * 2 - 3) as u64) as i32;
                    new_partitions.push((px, py, pw, split));
                    new_partitions.push((px, py + split, pw, ph - split));
                } else {
                    new_partitions.push((px, py, pw, ph));
                }
            }
            partitions = new_partitions;
        }

        // Create rooms within each partition
        for (px, py, pw, ph) in &partitions {
            let margin = 2;
            if *pw <= min_room_size + margin * 2 || *ph <= min_room_size + margin * 2 {
                continue;
            }
            let rw = min_room_size
                + (lcg_next(&mut rng_state) % (*pw - min_room_size - margin * 2 + 1) as u64) as i32;
            let rh = min_room_size
                + (lcg_next(&mut rng_state) % (*ph - min_room_size - margin * 2 + 1) as u64) as i32;
            let rx = px
                + margin
                + (lcg_next(&mut rng_state) % (*pw - rw - margin * 2 + 1) as u64) as i32;
            let ry = py
                + margin
                + (lcg_next(&mut rng_state) % (*ph - rh - margin * 2 + 1) as u64) as i32;
            rooms.push(Room::new(rx, ry, rw, rh));
        }

        // Connect rooms sequentially (guarantees all reachable)
        let mut corridors = Vec::new();
        for i in 1..rooms.len() {
            let (ax, ay) = rooms[i - 1].center();
            let (bx, by) = rooms[i].center();
            corridors.push(Corridor {
                from: (ax, ay),
                to: (bx, by),
            });
        }

        // Build tile grid
        let mut tiles = vec![Tile::Wall; (width * height) as usize];

        // Carve rooms
        for room in &rooms {
            for ry in room.y..(room.y + room.height).min(height) {
                for rx in room.x..(room.x + room.width).min(width) {
                    if rx >= 0 && ry >= 0 {
                        tiles[(ry * width + rx) as usize] = Tile::Floor;
                    }
                }
            }
        }

        // Carve corridors (L-shaped)
        for corridor in &corridors {
            let (x1, y1) = corridor.from;
            let (x2, y2) = corridor.to;

            // Horizontal segment
            let (sx, ex) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
            for x in sx..=ex {
                if x >= 0 && x < width && y1 >= 0 && y1 < height {
                    let idx = (y1 * width + x) as usize;
                    if tiles[idx] == Tile::Wall {
                        tiles[idx] = Tile::Corridor;
                    }
                }
            }

            // Vertical segment
            let (sy, ey) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
            for y in sy..=ey {
                if x2 >= 0 && x2 < width && y >= 0 && y < height {
                    let idx = (y * width + x2) as usize;
                    if tiles[idx] == Tile::Wall {
                        tiles[idx] = Tile::Corridor;
                    }
                }
            }
        }

        Self {
            width,
            height,
            rooms,
            corridors,
            tiles,
        }
    }

    /// Get tile at position.
    pub fn get_tile(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            Tile::Wall
        } else {
            self.tiles[(y * self.width + x) as usize]
        }
    }

    /// Count floor tiles (rooms + corridors).
    pub fn floor_count(&self) -> usize {
        self.tiles
            .iter()
            .filter(|t| **t == Tile::Floor || **t == Tile::Corridor)
            .count()
    }
}

/// Simple LCG PRNG.
fn lcg_next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1);
    *state >> 33
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dungeon_generates_rooms() {
        let d = Dungeon::generate(80, 50, 42, 5);
        assert!(!d.rooms.is_empty(), "should generate at least one room");
        assert!(d.rooms.len() >= 2, "should generate multiple rooms");
    }

    #[test]
    fn dungeon_is_deterministic() {
        let d1 = Dungeon::generate(80, 50, 42, 5);
        let d2 = Dungeon::generate(80, 50, 42, 5);
        assert_eq!(d1.rooms.len(), d2.rooms.len());
        assert_eq!(d1.tiles, d2.tiles);
    }

    #[test]
    fn dungeon_has_corridors() {
        let d = Dungeon::generate(80, 50, 42, 5);
        assert!(!d.corridors.is_empty());
        // Corridors = rooms - 1 (sequential connection)
        assert_eq!(d.corridors.len(), d.rooms.len() - 1);
    }

    #[test]
    fn dungeon_has_floor() {
        let d = Dungeon::generate(80, 50, 42, 5);
        assert!(d.floor_count() > 0);
    }

    #[test]
    fn room_center() {
        let r = Room::new(10, 20, 6, 8);
        assert_eq!(r.center(), (13, 24));
    }

    #[test]
    fn room_intersects() {
        let a = Room::new(0, 0, 10, 10);
        let b = Room::new(5, 5, 10, 10);
        let c = Room::new(20, 20, 5, 5);
        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn different_seeds_different_dungeons() {
        let d1 = Dungeon::generate(80, 50, 1, 5);
        let d2 = Dungeon::generate(80, 50, 99, 5);
        assert_ne!(d1.tiles, d2.tiles);
    }

    #[test]
    fn get_tile_out_of_bounds_is_wall() {
        let d = Dungeon::generate(40, 30, 42, 4);
        assert_eq!(d.get_tile(-1, 0), Tile::Wall);
        assert_eq!(d.get_tile(0, -1), Tile::Wall);
        assert_eq!(d.get_tile(40, 0), Tile::Wall);
    }
}
