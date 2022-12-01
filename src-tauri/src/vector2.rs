use std::f32::consts::PI;

#[derive(Clone, Copy, serde::Serialize)]
pub struct Vector2 {
	pub x: f32,
	pub y: f32,
}

impl Vector2 {
	fn length(self) -> f32 {
		return (self.x.powf(2.0) + self.y.powf(2.0)).sqrt();
	}
	fn angle(self) -> f32 {
		if self.x == 0.0 && self.y == 0.0 {
			return 0.0;	
		}
		let polar = 2.0*PI-(self.y).atan2(self.x);
		if polar+0.5*PI > 2.0*PI {
			return polar+0.5*PI - 2.0*PI;
		} else {
			return polar+0.5*PI;
		}
	}
	fn row(self, rows: usize) -> usize {
		return (((self.length().min(0.99)) * (rows+1) as f32).floor() - 1.0).max(0.0) as usize;
	}
	fn col(self, cols: usize) -> usize {
		return ((self.angle()/(2.0*PI))*cols as f32).floor() as usize;
	}
	pub fn selected(self, rows:usize, cols:usize) -> usize {
		return self.col(cols) + self.row(rows) * cols;
	}
	pub fn normalize(mut self) -> Vector2{
		if !(self.x == 0.0 && self.y == 0.0) {
			let len = self.length();
			self.x = self.x / len;
			self.y = self.y / len;
		}
		return self;
	}
}