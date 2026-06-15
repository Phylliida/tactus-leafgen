//! 2D vector over [`Scalar`](crate::Scalar).
//!
//! Kept deliberately small and method-based (rather than leaning on operator
//! overloading) so that the eventual generalization to a `Scalar` trait — for
//! the verified build — is a mechanical refactor.

use crate::Scalar;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: Scalar,
    pub y: Scalar,
}

impl Vec2 {
    pub fn new(x: Scalar, y: Scalar) -> Self {
        Vec2 { x, y }
    }

    pub fn zero() -> Self {
        Vec2::new(0.0, 0.0)
    }

    pub fn add(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x + o.x, self.y + o.y)
    }

    pub fn sub(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x - o.x, self.y - o.y)
    }

    pub fn scale(self, s: Scalar) -> Vec2 {
        Vec2::new(self.x * s, self.y * s)
    }

    pub fn dot(self, o: Vec2) -> Scalar {
        self.x * o.x + self.y * o.y
    }

    /// Squared length — preferred for comparisons (no `sqrt`, which is the op
    /// that will complicate the exact-arithmetic verified build).
    pub fn len_sq(self) -> Scalar {
        self.dot(self)
    }

    pub fn len(self) -> Scalar {
        self.len_sq().sqrt()
    }

    /// Unit vector, or zero for (near-)zero input.
    pub fn normalized(self) -> Vec2 {
        let l = self.len();
        if l > 1e-12 {
            self.scale(1.0 / l)
        } else {
            Vec2::zero()
        }
    }

    pub fn dist_sq(self, o: Vec2) -> Scalar {
        self.sub(o).len_sq()
    }
}
