use bevy::prelude::*;
use noise::Perlin;
use noise::NoiseFn;


pub struct Chunk {
    size: usize,
    chunk_coords_x: i32,
    chunk_coords_z: i32,
}

impl Chunk {
    pub fn new(size: usize, chunk_coords_x: i32,chunk_coords_z: i32,) -> Self {
        Chunk {
            size,
            chunk_coords_x,
            chunk_coords_z,
            
        }
        
    }
}

impl Chunk {
    pub fn create_chunk_mesh(&self, 
        commands: &mut Commands, 
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>
    ) -> Vec<Entity> {
        let cube_mesh = meshes.add(Cuboid::mesh(&Cuboid::new(1.0, 1.0, 1.0)));
        let cube_material = materials.add(Color::srgba_u8(30, 112, 0, 0));
        
        let x_chunk_origin: f32 = self.chunk_coords_x as f32 * self.size as f32;
        let z_chunk_origin: f32 = self.chunk_coords_z as f32 * self.size as f32;
        
        let mut entities: Vec<Entity> = Vec::new();
        for x in 0..self.size {
            for z in 0..self.size {
                let entity = commands.spawn(PbrBundle {
                    mesh: cube_mesh.clone(),
                    material: cube_material.clone(),
                    transform: Transform::from_xyz(
                        x_chunk_origin + x as f32, 
                        generate_height(14,  x_chunk_origin as f64 + x as f64, z_chunk_origin as f64 + z as f64, 0.001, 4) as i32 as f32, 
                        z_chunk_origin + z as f32
                    ),
                    ..default()
                }).id();
                
                entities.push(entity);
            }
        }
        
        entities
    }
}

// Bruit de perlin 2d
fn generate_height(seed: u32, x: f64, z: f64, scale: f64, octaves: u32) -> f32 {
    let perlin = Perlin::new(seed);
    
    let frequency = 16.0 * scale;
    let amplitude = 20.0 / (octaves as f64);
    
    let mut height = 0.0;
    for i in 0..octaves {
        let freq = frequency * (i as f64 + 1.0).powf(2.0);
        let amp = amplitude / (i as f64 + 1.0).powf(2.0);
        
        height += perlin.get([x * freq, z * freq]) * amp;
    }
    
    height as f32
}

