
pub struct PrefabRoom {
    pub template: &'static str,
    pub width: usize,
    pub height: usize,
    pub first_depth: i32,
    pub last_depth: i32
}

pub const TRAP: PrefabRoom = PrefabRoom {
    template: TRAP_MAP,
    width: 5,
    height: 5,
    first_depth: 0,
    last_depth: 100
};

pub const CHICKFILA: PrefabRoom = PrefabRoom {
    template: CHICKEN_SANDWICH,
    width: 3,
    height: 1,
    first_depth: 0,
    last_depth: 100
};

pub const CHECKERBOARD: PrefabRoom = PrefabRoom {
    template: CHECKERBOARD_MAP,
    width: 6,
    height: 5,
    first_depth: 0,
    last_depth: 100
};

pub const WELL: PrefabRoom = PrefabRoom {
    template: WELL_MAP,
    width: 7,
    height: 5,
    first_depth: 0,
    last_depth: 100
};

const TRAP_MAP: &str = " ^^^  ^!^  ^^^ ";

const CHICKEN_SANDWICH: &str = "=q=";

const CHECKERBOARD_MAP: &str = "
......
.o#o#.
.#^#u.
.!#.#.
......
";

const WELL_MAP: &str = "
.......
.##.##.
.#...#.
.##.##.
.......
";
