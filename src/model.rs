use std::{
    iter::Sum,
    ops::{Add, Div, Mul, Neg},
    sync::{Arc, Weak},
};

use crossbeam::sync::ShardedLock;

#[derive(Copy, Clone, Debug)]
pub struct Vector2D {
    x: f32,
    y: f32,
}

impl Vector2D {
    const ZERO: Self = Vector2D { x: 0.0, y: 0.0 };

    #[inline(always)]
    fn scale(self, lambda: f32) -> Self {
        Self {
            x: self.x * lambda,
            y: self.y * lambda,
        }
    }

    #[inline(always)]
    pub fn length(self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    #[inline(always)]
    fn normalize(self) -> Self {
        let length = self.length();
        self / length
    }

    #[inline(always)]
    fn travel(self, t: f32) -> Self {
        self * 0.5 * t.powi(2)
    }
}

impl Add for Vector2D {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Mul<f32> for Vector2D {
    type Output = Vector2D;
    #[inline(always)]
    fn mul(self, rhs: f32) -> Self::Output {
        self.scale(rhs)
    }
}

impl Div<f32> for Vector2D {
    type Output = Vector2D;
    #[inline(always)]
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Neg for Vector2D {
    type Output = Vector2D;
    #[inline(always)]
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Sum for Vector2D {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Vector2D::ZERO, |acc, x| acc + x)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Coordinates {
    pub x: f32,
    pub y: f32,
}

impl Coordinates {
    pub fn to(self, other: Self) -> Vector2D {
        Vector2D {
            x: other.x - self.x,
            y: other.y - self.y,
        }
    }
}

impl Add<Vector2D> for Coordinates {
    type Output = Coordinates;
    fn add(self, rhs: Vector2D) -> Self::Output {
        Coordinates {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[derive(Debug)]
pub struct Node {
    id: usize,
    pub loc: ShardedLock<Coordinates>,
    pub weight: f32,
    from: ShardedLock<Vec<Weak<Relation>>>,
    to: ShardedLock<Vec<Weak<Relation>>>,
}

impl Node {
    pub fn new(id: usize, x: f32, y: f32, weight: f32) -> Self {
        Self {
            id,
            loc: ShardedLock::new(Coordinates { x, y }),
            weight,
            from: ShardedLock::new(Vec::new()),
            to: ShardedLock::new(Vec::new()),
        }
    }

    pub fn calc_new_position(
        &self,
        other: &[Arc<Self>],
        spring_scale: f32,
        coloumb_scale: f32,
        t: f32,
    ) -> Coordinates {
        let offset = self.compound_vector(other, spring_scale, coloumb_scale);
        *self.loc.read().unwrap() + offset.travel(t)
    }

    pub fn update_coordinates(&self, new: Coordinates) {
        let mut m = self.loc.write().unwrap();
        *m = new;
    }

    #[inline(always)]
    fn distance_squared(&self, other: &Self) -> f32 {
        let (self_x, self_y) = {
            let guard = self.loc.read().expect("Lock is poisoned");
            (guard.x, guard.y)
        };
        let (other_x, other_y) = {
            let guard = other.loc.read().expect("Lock is poisoned");
            (guard.x, guard.y)
        };
        let x_delta = (self_x - other_x).abs().powi(2);
        let y_delta = (self_y - other_y).abs().powi(2);
        x_delta + y_delta
    }

    #[inline(always)]
    fn coloumb_force(&self, other: &Self, scale: f32) -> f32 {
        scale * (self.weight * other.weight) / self.distance_squared(other)
    }

    #[inline(always)]
    pub fn coloumb_vector(&self, other: &Self, scale: f32) -> Vector2D {
        let force = self.coloumb_force(other, scale);
        let direction = -self
            .loc
            .read()
            .unwrap()
            .to(*other.loc.read().unwrap())
            .normalize();
        direction * force
    }

    #[inline(always)]
    fn spring_vector(&self, scale: f32) -> Vector2D {
        let from_guard = self.from.read().unwrap();
        let to_guard = self.to.read().unwrap();
        let from_iter = from_guard
            .iter()
            .map(|e| e.upgrade().unwrap().hook_vector(scale));
        let to_iter = to_guard
            .iter()
            .map(|e| -e.upgrade().unwrap().hook_vector(scale));

        from_iter.chain(to_iter).sum()
    }

    #[inline(always)]
    fn compound_vector(
        &self,
        other: &[Arc<Self>],
        spring_scale: f32,
        coloumb_scale: f32,
    ) -> Vector2D {
        let tmp: Vector2D = other
            .iter()
            .filter(|e| e.id != self.id)
            .map(|e| self.coloumb_vector(e, coloumb_scale))
            .sum();
        tmp + self.spring_vector(spring_scale)
    }
}

#[derive(Debug)]
pub struct Relation {
    pub weight_squared: f32,
    pub from: Arc<Node>,
    pub to: Arc<Node>,
}

impl Relation {
    pub fn new(weight: f32, from: Arc<Node>, to: Arc<Node>) -> Self {
        Self {
            weight_squared: weight.powi(2),
            from,
            to,
        }
    }

    #[inline(always)]
    fn distance_squared(&self) -> f32 {
        self.from.distance_squared(&self.to)
    }

    #[inline(always)]
    fn hook_force_squared(&self, scale: f32) -> f32 {
        let stretch = self.distance_squared();
        self.weight_squared * stretch * scale
    }

    #[inline(always)]
    fn hook_force(&self, scale: f32) -> f32 {
        self.hook_force_squared(scale).sqrt()
    }

    #[inline(always)]
    fn hook_vector(&self, scale: f32) -> Vector2D {
        let force = self.hook_force(scale);
        let direction = self
            .from
            .loc
            .read()
            .unwrap()
            .to(*self.to.loc.read().unwrap())
            .normalize();
        direction * force
    }

    pub fn register(self: &Arc<Self>) {
        let weak_from = Arc::downgrade(self);
        let weak_to = Arc::downgrade(self);
        self.from.from.write().unwrap().push(weak_from);
        self.to.to.write().unwrap().push(weak_to);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{Node, Relation};

    #[test]
    fn relation_distance() {
        let from = Arc::new(Node::new(1, 0.0, 0.0, 1.0));
        let to = Arc::new(Node::new(1, 2.0, 2.0, 1.0));
        let relation = Relation::new(1.0, from, to);
        let distance = relation.distance_squared();
        let difference = (distance - 8.0).abs();
        assert!(
            difference < 1.0E-10,
            "Difference is {difference} Distance => {distance}"
        );
    }

    #[test]
    fn relation_hook_force() {
        let from = Arc::new(Node::new(1, 0.0, 0.0, 1.0));
        let to = Arc::new(Node::new(1, 2.0, 2.0, 1.0));
        let relation = Relation::new(1.0, from, to);
        let force = relation.hook_force(1.0);
        assert_eq!(force, 8.0f32.sqrt());

        let force_vector = relation.hook_vector(1.0);
        assert_eq!(force_vector.length(), 8.0f32.sqrt());
        assert_eq!(force_vector.x, 2.0);
        assert_eq!(force_vector.y, 2.0);
    }

    #[test]
    fn coordinates_calculation() {
        let from = Arc::new(Node::new(1, 0.0, 0.0, 1.0));
        let to = Arc::new(Node::new(2, 2.0, 2.0, 1.0));
        let nodes = Vec::from([from.clone(), to.clone()]);
        let r = Arc::new(Relation::new(1.0, from, to));
        r.register();
        let new_coordinates = nodes[0].calc_new_position(&nodes, 1.0, 1.0, 1.0);
        dbg!(new_coordinates);
    }
}
