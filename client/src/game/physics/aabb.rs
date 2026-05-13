use cgmath::{Point3, Vector3};

/// Boîte englobante alignée sur les axes (Axis-Aligned Bounding Box).
/// `min` est le coin inférieur (petites valeurs), `max` le coin supérieur (grandes valeurs).
pub struct AABB {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}
impl AABB {
    /// Construit un AABB centré en `center` avec une demi-taille `half_size`.
    /// Le volume total est donc `(2×half_size)³`.
    pub fn new(center: Point3<f32>, half_size: f32) -> Self {
        Self {
            min: Vector3::new(center.x - half_size, center.y - half_size, center.z - half_size),
            max: Vector3::new(center.x + half_size, center.y + half_size, center.z + half_size),
        }
    }

    /// Déplace l'AABB d'un vecteur `v`.
    pub fn translate(&mut self, v: Vector3<f32>) {
        self.max += v;
        self.min += v;
    }

    /// Teste si cet AABB chevauche un autre AABB (intersection non vide).
    /// Vérifie la séparation sur chacun des 3 axes.
    pub fn overlaps(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Retourne les 8 sommets de la boîte (utile pour le débogage).
    pub fn corners(&self) -> [Vector3<f32>; 8] {
        let [min, max] = [&self.min, &self.max];
        [
            Vector3::new(min.x, min.y, min.z),
            Vector3::new(max.x, min.y, min.z),
            Vector3::new(min.x, max.y, min.z),
            Vector3::new(max.x, max.y, min.z),
            Vector3::new(min.x, min.y, max.z),
            Vector3::new(max.x, min.y, max.z),
            Vector3::new(min.x, max.y, max.z),
            Vector3::new(max.x, max.y, max.z),
        ]
    }
}
