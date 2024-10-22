use bevy::math::VectorSpace;
use bevy::prelude::*;
use bevy::render::render_resource::encase::internal::CreateFrom;
use noise::Perlin;
use noise::NoiseFn;
use rayon::vec;

use super::ChunkWidth;

pub struct Chunk {
    size: usize,
    chunk_coords_x: i32,
    chunk_coords_z: i32,
}

enum Blocks {
    Air = 0,
    Grass = 1
}

impl Chunk {
    pub fn new(size: usize, chunk_coords_x: i32,chunk_coords_z: i32,) -> Self {
        Chunk {
            size,
            chunk_coords_x,
            chunk_coords_z,           
        }       
    }

    pub fn create_chunk_mesh(&self, 
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>
    ) -> Vec<Entity> {

        let triangle_mesh = meshes.add(Triangle3d::mesh(&Triangle3d::new(Vec3::X, Vec3::Y, Vec3::Z)));
        let triangle_x_1 = meshes.add(Triangle3d::mesh(&Triangle3d::new(Vec3::X, Vec3::Y, Vec3::ZERO)));
        let triangle_x_2 = meshes.add(Triangle3d::mesh(&Triangle3d::new(Vec3::Y+Vec3::X, Vec3::Y, Vec3::X)));
        let triangle_x_1_r = meshes.add(Triangle3d::mesh(&Triangle3d::new(-Vec3::X, Vec3::Y, Vec3::ZERO)));
        let triangle_x_2_r = meshes.add(Triangle3d::mesh(&Triangle3d::new(Vec3::Y-Vec3::X, Vec3::Y, -Vec3::X)));
        let triangle_y_1 = meshes.add(Triangle3d::mesh(&Triangle3d::new(Vec3::X, Vec3::ZERO, Vec3::Z)));
        let triangle_y_2 = meshes.add(Triangle3d::mesh(&Triangle3d::new(-Vec3::X, Vec3::ZERO, -Vec3::Z)));
        let triangle_z_1 = meshes.add(Triangle3d::mesh(&Triangle3d::new(Vec3::Z, Vec3::Y, Vec3::ZERO)));
        let triangle_z_2 = meshes.add(Triangle3d::mesh(&Triangle3d::new(Vec3::Y+Vec3::Z, Vec3::Y, Vec3::Z)));
        let triangle_z_1_r = meshes.add(Triangle3d::mesh(&Triangle3d::new(-Vec3::Z, Vec3::Y, Vec3::ZERO)));
        let triangle_z_2_r = meshes.add(Triangle3d::mesh(&Triangle3d::new(Vec3::Y-Vec3::Z, Vec3::Y, -Vec3::Z)));
        let triangle_material = materials.add(Color::srgba_u8(127, 127, 127, 255));

        let x_chunk_origin: f32 = self.chunk_coords_x as f32 * self.size as f32;
        let z_chunk_origin: f32 = self.chunk_coords_z as f32 * self.size as f32;

        let mut entities: Vec<Entity> = Vec::new();

        let mut a: Vec<Vec<(i32, i32)>> = vec![vec![(0, 0); self.size]; self.size];

        for x in 0..self.size {
            for z in 0..self.size {
                a[x][z] = (generate_height(14, x_chunk_origin as f64 + x as f64, z_chunk_origin as f64 + z as f64, 0.001, 4) as i32, 0);
            }
        }

        for x in 0..self.size {
            for z in 0..self.size {
                let y = a[x][z].0;

                if x > 0 && a[x-1][z].0 < y {
                    let entity1 = commands.spawn(PbrBundle {
                        mesh: triangle_z_1.clone(),
                        material: triangle_material.clone(),
                        transform: Transform::from_xyz(
                            x_chunk_origin - 0.5 + x as f32, 
                            y as f32 - 0.5, 
                            z_chunk_origin - 0.5 + z as f32
                        ),
                        ..default()
                    }).id();
                    entities.push(entity1);
                    let entity2 = commands.spawn(PbrBundle {
                        mesh: triangle_z_2.clone(),
                        material: triangle_material.clone(),
                        transform: Transform::from_xyz(
                            x_chunk_origin - 0.5 + x as f32, 
                            y as f32 - 0.5, 
                            z_chunk_origin - 0.5 + z as f32
                        ),
                        ..default()
                    }).id();
                    entities.push(entity2);
                }
                if x < self.size-1 && a[x+1][z].0 < y {
                    let entity1 = commands.spawn(PbrBundle {
                        mesh: triangle_z_1_r.clone(),
                        material: triangle_material.clone(),
                        transform: Transform::from_xyz(
                            x_chunk_origin + 0.5 + x as f32, 
                            y as f32 - 0.5, 
                            z_chunk_origin + 0.5 + z as f32
                        ),
                        ..default()
                    }).id();
                    entities.push(entity1);
                    let entity2 = commands.spawn(PbrBundle {
                        mesh: triangle_z_2_r.clone(),
                        material: triangle_material.clone(),
                        transform: Transform::from_xyz(
                            x_chunk_origin + 0.5 + x as f32, 
                            y as f32 - 0.5, 
                            z_chunk_origin + 0.5 + z as f32
                        ),
                        ..default()
                    }).id();
                    entities.push(entity2);
                }
                if z > 0 && a[x][z-1].0 < y {
                    let entity1 = commands.spawn(PbrBundle {
                        mesh: triangle_x_1_r.clone(),
                        material: triangle_material.clone(),
                        transform: Transform::from_xyz(
                            x_chunk_origin + 0.5 + x as f32, 
                            y as f32 - 0.5, 
                            z_chunk_origin - 0.5 + z as f32
                        ),
                        ..default()
                    }).id();
                    entities.push(entity1);
                    let entity2 = commands.spawn(PbrBundle {
                        mesh: triangle_x_2_r.clone(),
                        material: triangle_material.clone(),
                        transform: Transform::from_xyz(
                            x_chunk_origin + 0.5 + x as f32, 
                            y as f32 - 0.5, 
                            z_chunk_origin - 0.5 + z as f32
                        ),
                        ..default()
                    }).id();
                    entities.push(entity2);
                }
                if z < self.size-1 && a[x][z+1].0 < y {
                    let entity1 = commands.spawn(PbrBundle {
                        mesh: triangle_x_1.clone(),
                        material: triangle_material.clone(),
                        transform: Transform::from_xyz(
                            x_chunk_origin - 0.5 + x as f32, 
                            y as f32 - 0.5, 
                            z_chunk_origin + 0.5 + z as f32
                        ),
                        ..default()
                    }).id();
                    entities.push(entity1);
                    let entity2 = commands.spawn(PbrBundle {
                        mesh: triangle_x_2.clone(),
                        material: triangle_material.clone(),
                        transform: Transform::from_xyz(
                            x_chunk_origin - 0.5 + x as f32, 
                            y as f32 - 0.5, 
                            z_chunk_origin + 0.5 + z as f32
                        ),
                        ..default()
                    }).id();
                    entities.push(entity2);
                }
                let entity = commands.spawn(PbrBundle {
                    mesh: triangle_y_1.clone(),
                    material: triangle_material.clone(),
                    transform: Transform::from_xyz(
                        x_chunk_origin - 0.5 + x as f32, 
                        y as f32 + 0.5, 
                        z_chunk_origin - 0.5 + z as f32
                    ),
                    ..default()
                }).id();
                entities.push(entity);
                let entity2 = commands.spawn(PbrBundle {
                    mesh: triangle_y_2.clone(),
                    material: triangle_material.clone(),
                    transform: Transform::from_xyz(
                        x_chunk_origin + 0.5 + x as f32, 
                        y as f32 + 0.5, 
                        z_chunk_origin + 0.5 + z as f32
                    ),
                    ..default()
                }).id();
                entities.push(entity2);
            }
        }

        entities

        /*let cube_mesh = meshes.add(Cuboid::mesh(&Cuboid::new(1.0, 1.0, 1.0)));
        let cube_material = materials.add(Color::srgba_u8(30, 112, 0, 255));
        
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
        
        entities*/
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

