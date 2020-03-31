pub struct Rect {
    pub x1: i32,
    pub x2: i32, // width
    pub y1: i32,
    pub y2: i32, // height
}

impl Rect {
    // return new instance of a rectanlgle <Constructor>
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Rect {
        Rect {
            x1: x,
            y1: y,
            x2: x + width,
            y2: y + height,
        }
    }

    // check if rectangle is drawn upon 'other' rectangle <Member>
    pub fn intersect(&self, other: &Rect) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }

    // return center position of given rectangle. <Member>
    // helps with drawing the corridors between rooms
    pub fn center(&self) -> (i32, i32) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }
}
