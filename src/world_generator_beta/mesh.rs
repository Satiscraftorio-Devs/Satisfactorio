use binary_greedy_meshing as bgm;
use std::collections::BTreeSet;
use bevy::prelude::*;
use bevy::render::{
    mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
    render_asset::RenderAssetUsages,
};
use super::perlin::*;

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

const C_SZ: usize = 64;
const C_PD: usize = 2;
const C_FS: usize = C_SZ;

fn get_blocks() -> [[[bool; C_FS]; C_FS]; C_FS] {
    let mut blocks: [[[bool; C_FS]; C_FS]; C_FS] = [[[false; C_FS]; C_FS]; C_FS];

    for x in 0..C_SZ {
        for z in 0..C_SZ {
            let y = generate_height(14, x as f64, z as f64, 0.001, 2) as usize;
            for w in 0..y {
                blocks[x][w][z] = true;
            }
        }
    }

    blocks
}

fn get_triangles_axis(blocks: [[bool; C_FS]; C_FS]) -> Vec<[Vec3; 3]> {
    let triangles: Vec<[Vec3; 3]> = Vec::new();

    let mut x = 0;
    for y in blocks {
        let mut w: usize = 0;
        let mut face_corners_pos: [Vec2; 2] = [Vec2::ZERO; 2];
        let mut first_pos_index: usize = 0;
        while w < C_FS-1 {
            if y[w] == true && face_corners_pos[0] == Vec2::ZERO {
                face_corners_pos[0] = Vec2::new(x as f32, w as f32);
                first_pos_index = w;
            }
            else if y[w] == true {
                face_corners_pos[1] = Vec2::new(x as f32, w as f32);
            }
            else if y[w] == false && x < C_FS-1 {
                let mut z = x;
                while z < C_FS-1 {
                    let v = &(blocks[z+1][first_pos_index..w-1]);
                    if v.contains(&false) {
                        face_corners_pos[1] = Vec2::new(z as f32, (w-1) as f32);
                        break;
                    }
                    else {
                        z+=1;
                    }
                }
            }
            w += 1;
        }
        x += 1;
    }

    triangles
}