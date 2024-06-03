use eframe::epaint::Pos2;

pub struct Square {
    pub pos: Pos2,
    pub size: Pos2,
    pub color: [f32; 3],
    pub radius: f32,
}

impl Default for Square {
    fn default() -> Self {
        Self {
            pos: Pos2 { x: 50.0, y: 50.0 },
            size: Pos2 { x: 100.0, y: 100.0 },
            color: [1.0, 1.0, 1.0],
            radius: 0.2,
        }
    }
}

pub struct Circle {
    pub pos: Pos2,
    pub radius: f32,
    pub color: [f32; 3],
}

impl Default for Circle {
    fn default() -> Self {
        Self {
            pos: Pos2 { x: 0.0, y: 0.0 },
            radius: 100.0,
            color: [1.0, 1.0, 1.0],
        }
    }
}

pub enum Shape {
    Square(Square, f32, f32, bool),
    Circle(Circle, f32, f32, bool),
}

impl Shape {
    pub fn default_square() -> Self {
        Self::Square(Square::default(), 0.5, 0.5, false)
    }

    pub fn default_circle() -> Self {
        Self::Circle(Circle::default(), 0.0, 0.0, false)
    }
}
