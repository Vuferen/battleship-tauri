struct Vector2 {
	x: f32,
	y: f32,
}

impl Vector2 {
	fn length() {
		return (x.pow(2) + y.pow(2)).sqrt();
	}
	fn angle() {
		(y/x).atan() + PI*0.5;
	}
	pub fn col(self, cols: usize) -> usize {
		return 1-self.length();
	}
	pub fn row(self, rows: usize) -> usize {
		return floor((self.angle()/(2*PI))*rows);
	}
}