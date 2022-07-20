use crate::model::{Node, Relation};

pub struct Renderer {
    elements: Vec<Element>,
    bounds: Bounds,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            bounds: Bounds {
                min_x: 0.0,
                max_x: 0.0,
                min_y: 0.0,
                max_y: 0.0,
            },
        }
    }

    pub fn add_element(&mut self, element: Element) {
        let bounds = element.bounds();
        self.bounds.update(bounds);
        self.elements.push(element);
    }

    pub fn render(self, x: f32, y: f32) -> String {
        let outer_x = self.bounds.max_x - self.bounds.min_x;
        let outer_y = self.bounds.max_y - self.bounds.min_y;

        debug_assert!(outer_x.is_sign_positive());
        debug_assert!(outer_y.is_sign_positive());

        let x_scale = x / outer_x;
        let y_scale = y / outer_y;

        let inner_svg_parts: Vec<String> = self
            .elements
            .iter()
            .map(|e| e.render(x_scale, self.bounds.min_x, y_scale, self.bounds.min_y))
            .collect();

        let inner_svg = inner_svg_parts.join("\n");

        format!(
            r#"
        <svg width="{x}" height="{y}" xmlns="http://www.w3.org/2000/svg">
        {inner_svg}
        </svg>
        "#
        )
    }
}

pub enum Element {
    Circle { radius: f32, x: f32, y: f32 },
    Line { start: (f32, f32), stop: (f32, f32) },
    Tag { content: String, x: f32, y: f32 },
}

struct Bounds {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

impl Bounds {
    fn update(&mut self, new: Bounds) {
        self.min_x = self.min_x.min(new.min_x);
        self.max_x = self.max_x.max(new.max_x);
        self.min_y = self.min_y.min(new.min_y);
        self.max_y = self.max_y.max(new.max_y);
    }
}

impl Element {
    fn bounds(&self) -> Bounds {
        match self {
            Element::Circle { radius, x, y } => Bounds {
                min_x: x - radius,
                max_x: x + radius,
                min_y: y - radius,
                max_y: y + radius,
            },
            Element::Line { start, stop } => Bounds {
                min_x: start.0.min(stop.0),
                max_x: start.0.max(stop.0),
                min_y: start.1.min(stop.1),
                max_y: start.1.max(stop.1),
            },
            Element::Tag { x, y, .. } => Bounds {
                min_x: *x,
                max_x: *x,
                min_y: *y,
                max_y: *y,
            },
        }
    }

    pub fn render(&self, x_scale: f32, x_offset: f32, y_scale: f32, y_offset: f32) -> String {
        match self {
            Element::Circle { radius, x, y } => {
                let (new_x, new_y) = ((x - x_offset) * x_scale, (y - y_offset) * y_scale);
                format!(r#"<circle stroke="black" stroke-width="2px" cx="{new_x}" cy="{new_y}" r="{radius}" />"#)
            }
            Element::Line { start, stop } => {
                let (x1, y1) = (
                    (start.0 - x_offset) * x_scale,
                    (start.1 - y_offset) * y_scale,
                );
                let (x2, y2) = ((stop.0 - x_offset) * x_scale, (stop.1 - y_offset) * y_scale);
                format!(r#"<line stroke="black" stroke-width="2px" x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" />"#)
            }
            Element::Tag { .. } => todo!(),
        }
    }
}

impl From<&Node> for Element {
    fn from(n: &Node) -> Self {

        let m = n.loc.read().unwrap().clone();
        Element::Circle {
            radius: n.weight ,
            x: m.x,
            y: m.y,
        }
    }
}

impl From<&Relation> for Element {
    fn from(r: &Relation) -> Self {
        let from = r.from.loc.read().unwrap().clone();
        let to = r.to.loc.read().unwrap().clone();
        Element::Line {
            start: (from.x, from.y),
            stop: (to.x, to.y),
        }
    }
}
