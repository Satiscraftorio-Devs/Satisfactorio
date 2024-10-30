use bevy::prelude::*;
use bevy::render::{
    mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
    render_asset::RenderAssetUsages,
};

use noise::Perlin;
use noise::NoiseFn;
use std::collections::BTreeSet;
use binary_greedy_meshing as bgm;

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

    pub fn create_chunk_mesh(&self, 
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>
    ) -> Vec<Entity> {
        
        let x_chunk_origin: f32 = self.chunk_coords_x as f32 * self.size as f32;
        let z_chunk_origin: f32 = self.chunk_coords_z as f32 * self.size as f32;
        
        let mut entities: Vec<Entity> = Vec::new();
        let mesh = meshes.add(generate_mesh(x_chunk_origin, z_chunk_origin));

        let entity = commands.spawn(PbrBundle {
            mesh,
            material: materials.add(StandardMaterial {
                base_color: Color::linear_rgba(0.1, 0.1, 0.1, 1.0),
                ..Default::default()
            }),
            transform: Transform::from_xyz(
                x_chunk_origin, 
                0.0,
                z_chunk_origin
                ),
            ..Default::default()
        }).id();

        entities.push(entity);
        entities

    }
}
        

// Bruit de perlin 2d
#[inline]
fn generate_height(seed: u32, x: f64, z: f64, scale: f64, octaves: i32) -> f32 {
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

fn generate_mesh(x_chunk_origin: f32, z_chunk_origin: f32) -> Mesh {
    const MASK6: u32 = 0b111_111;

    let voxels = voxel_buffer(x_chunk_origin, z_chunk_origin);
    let mut mesh_data = bgm::MeshData::new();

    bgm::mesh(&voxels, &mut mesh_data, BTreeSet::from([2, 3]));
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    for (face_n, quads) in mesh_data.quads.iter().enumerate() {
        let face: bgm::Face = (face_n as u8).into();
        let n = face.n();
        for quad in quads {
            let vertices_packed = face.vertices_packed(*quad);
            for vertex_packed in vertices_packed.iter() {
                let x = *vertex_packed & MASK6;
                let y = (*vertex_packed >> 6) & MASK6;
                let z = (*vertex_packed >> 12) & MASK6;
                positions.push([x as f32, y as f32, z as f32]);
                normals.push(n.clone());
            }
        }
    }
    let indices = bgm::indices(positions.len());
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::Float32x2(vec![[0.0; 2]; positions.len()]),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    mesh.insert_indices(Indices::U32(indices));
    mesh
    
}

#[inline]
fn voxel_buffer(x_chunk_origin: f32, z_chunk_origin: f32) -> [u16; bgm::CS_P3] {
    let mut voxels: [u16; 262144] = [0; bgm::CS_P3];
    for x in 0..bgm::CS {
            for z in 0..bgm::CS {
                let y = generate_height(14, x_chunk_origin as f64 + x as f64, z_chunk_origin as f64 + z as f64, 0.001, 2) as usize;
                voxels[bgm::pad_linearize(x, y, z)] = 1;
            }
    }
    voxels
}