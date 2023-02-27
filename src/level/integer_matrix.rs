use bevy::math::IVec3;

#[derive(Clone, Copy, Debug)]
pub struct IMat3 {
    pub columns: [IVec3; 3],
}

impl IMat3 {
    pub fn mul_vec3(&self, rhs: &IVec3) -> IVec3 {
        let x = IVec3::new(
            self.columns[0].x, self.columns[1].x, self.columns[2].x
        ).dot(*rhs);
        let y = IVec3::new(
            self.columns[0].y, self.columns[1].y, self.columns[2].y
        ).dot(*rhs);
        let z = IVec3::new(
            self.columns[0].z, self.columns[1].z, self.columns[2].z
        ).dot(*rhs);
        IVec3::new(x, y, z)
    }

    pub fn mul_mat3(&self, rhs: &IMat3) -> IMat3 {
        IMat3 {
            columns: [
                self.mul_vec3(&rhs.columns[0]),
                self.mul_vec3(&rhs.columns[1]),
                self.mul_vec3(&rhs.columns[2]),
            ],
        }
    }

    pub fn inverse(&self) -> Self {
        let mut tmp0 = self.columns[1].cross(self.columns[2]);
        let mut tmp1 = self.columns[2].cross(self.columns[0]);
        let mut tmp2 = self.columns[0].cross(self.columns[1]);
        let det = self.columns[2].dot(tmp2);
        assert_ne!(det, 0);
        assert_eq!(tmp0.x % det, 0);
        assert_eq!(tmp0.y % det, 0);
        assert_eq!(tmp0.z % det, 0);
        assert_eq!(tmp1.x % det, 0);
        assert_eq!(tmp1.y % det, 0);
        assert_eq!(tmp1.z % det, 0);
        assert_eq!(tmp2.x % det, 0);
        assert_eq!(tmp2.y % det, 0);
        assert_eq!(tmp2.z % det, 0);
        tmp0 /= det;
        tmp1 /= det;
        tmp2 /= det;
        IMat3 {
            columns: [
                IVec3::new(tmp0.x, tmp1.x, tmp2.x),
                IVec3::new(tmp0.y, tmp1.y, tmp2.y),
                IVec3::new(tmp0.z, tmp1.z, tmp2.z),
            ],
        }
    }
}
