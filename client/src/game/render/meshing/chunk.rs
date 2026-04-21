use crate::{
    engine::render::mesh::{manager::DataEntry, mesh::MeshId},
    game::{
        render::utils::{
            face_mask::FaceMask,
            padded_chunk::{LAST_PADDED_CHUNK_CENTER_INDEX, PADDED_CHUNK_BLOCK_NUMBER, PADDED_CHUNK_SIZE, PADDED_CHUNK_SIZE_SQR, PaddedChunk},
        },
        world::world::MeshSnapshot,
    },
};
use cgmath::Vector3;
use shared::world::data::chunk::{CHUNK_SIZE, CHUNK_SIZE_F, Chunk, LAST_CHUNK_AXIS_INDEX, LAST_CHUNK_AXIS_INDEX_USIZE};
use shared::{parallel::Parallelizable, time};
use std::sync::{
    Arc, atomic::{AtomicBool, Ordering}
};

use crate::{
    common::geometry::{direction::Direction, vertex::Vertex},
    engine::render::render::Renderer,
};

enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    #[inline(always)]
    pub const fn to_usize(self) -> usize{
        self as usize
    }
}

pub struct ChunkMesh {
    pub id: Option<MeshId>,
    dirty: AtomicBool,
}

const GREEDY_MESH_MAX_FACE_WIDTH: usize = CHUNK_SIZE as usize;
const GREEDY_MESH_MAX_FACE_HEIGHT: usize = CHUNK_SIZE as usize;

impl ChunkMesh {
    pub fn new() -> ChunkMesh {
        return ChunkMesh {
            id: None,
            dirty: AtomicBool::new(true),
        };
    }

    pub fn is_dirty(&self) -> bool {
        return self.dirty.load(Ordering::Relaxed);
    }

    pub fn get_v_ao(chunk: &PaddedChunk, pos: [i32; 3], neighbors: [(i32, i32, i32); 3]) -> u8 {
        // Check if neighbors exists
        let corner_solid = chunk
            .get_block_from_chunk_xyz(pos[0] + neighbors[0].0, pos[1] + neighbors[0].1, pos[2] + neighbors[0].2)
            .is_solid() as u8;
        let side1_solid = chunk
            .get_block_from_chunk_xyz(pos[0] + neighbors[1].0, pos[1] + neighbors[1].1, pos[2] + neighbors[1].2)
            .is_solid() as u8;
        let side2_solid = chunk
            .get_block_from_chunk_xyz(pos[0] + neighbors[2].0, pos[1] + neighbors[2].1, pos[2] + neighbors[2].2)
            .is_solid() as u8;

        // If the corner is surround by side1 and side2, it is necessarly opaque, whether corner is solid or not.
        if side1_solid == 1 && side2_solid == 1 {
            return 0;
        }
        // If not, each neighbor bring the light down equally
        else {
            return 3 - (side1_solid + side2_solid + corner_solid);
        }
    }

    pub fn get_v_unpadded_ao(chunk: &PaddedChunk, pos: [i32; 3], neighbors: [(i32, i32, i32); 3]) -> u8 {
        // Check if neighbors exists
        let corner_solid = chunk
            .get_block_from_xyz_unsafe(pos[0] + neighbors[0].0, pos[1] + neighbors[0].1, pos[2] + neighbors[0].2)
            .is_solid() as u8;
        let side1_solid = chunk
            .get_block_from_xyz_unsafe(pos[0] + neighbors[1].0, pos[1] + neighbors[1].1, pos[2] + neighbors[1].2)
            .is_solid() as u8;
        let side2_solid = chunk
            .get_block_from_xyz_unsafe(pos[0] + neighbors[2].0, pos[1] + neighbors[2].1, pos[2] + neighbors[2].2)
            .is_solid() as u8;

        // [side1][side2][corner]
        // [0][0][0] : 3
        // [1][0][0] : 2
        // [0][1][0] : 2
        // [1][1][0] : 0
        // [0][0][1] : 2
        // [1][0][1] : 1
        // [0][1][1] : 1
        // [1][1][1] : 0
        const AO_TABLE: [u8; 8] = [3, 2, 2, 0, 2, 1, 1, 0];
        let idx = ((corner_solid << 2) | (side1_solid << 1) | side2_solid) as usize;
        return AO_TABLE[idx];
    }

    /// Don't know how, but it's working though
    fn get_ao_offsets(face: Direction, corner: Corner) -> [(i32, i32, i32); 3] {
        let (u, v, normal) = match face {
            Direction::Left => ((0, 1, 0), (0, 0, 1), (-1, 0, 0)),
            Direction::Bottom => ((1, 0, 0), (0, 0, 1), (0, -1, 0)),
            Direction::Back => ((1, 0, 0), (0, 1, 0), (0, 0, -1)),
            Direction::Right => ((0, 1, 0), (0, 0, 1), (1, 0, 0)),
            Direction::Top => ((1, 0, 0), (0, 0, 1), (0, 1, 0)),
            Direction::Front => ((1, 0, 0), (0, 1, 0), (0, 0, 1)),
        };

        let (u, v) = match face {
            Direction::Top | Direction::Bottom => (u, v),
            _ => (v, u),
        };

        let (su, sv) = match corner {
            Corner::BottomLeft => (-1, -1),
            Corner::BottomRight => (1, -1),
            Corner::TopLeft => (-1, 1),
            Corner::TopRight => (1, 1),
        };

        let side1 = (
            u.0 * su + normal.0,
            u.1 * su + normal.1,
            u.2 * su + normal.2
        );
        let side2 = (
            v.0 * sv + normal.0,
            v.1 * sv + normal.1,
            v.2 * sv + normal.2
        );
        let corner = (
            side1.0 + side2.0 - normal.0,
            side1.1 + side2.1 - normal.1,
            side1.2 + side2.2 - normal.2,
        );

        [corner, side1, side2]
    }

    const fn get_ao_optimized_offsets(face: Direction, corner: Corner) -> [(i32, i32, i32); 3] {
        const UVNORMAL_TABLE: [((i32, i32, i32), (i32, i32, i32), (i32, i32, i32)); 6] = [
            ((0, 1, 0), (0, 0, 1), (-1, 0, 0)),
            ((1, 0, 0), (0, 0, 1), (0, -1, 0)),
            ((1, 0, 0), (0, 1, 0), (0, 0, -1)),
            ((0, 1, 0), (0, 0, 1), (1, 0, 0)),
            ((1, 0, 0), (0, 0, 1), (0, 1, 0)),
            ((1, 0, 0), (0, 1, 0), (0, 0, 1)),
        ];
        const CORNER_TABLE: [(i32, i32); 4] = [
            (-1, 1),
            (1, 1),
            (-1, -1),
            (1, -1),
        ];

        let (u, v, normal) = UVNORMAL_TABLE[face.to_usize()];
        let (su, sv) = CORNER_TABLE[corner.to_usize()];
        let (u, v) = match face {
            Direction::Top | Direction::Bottom => (u, v),
            _ => (v, u),
        };

        let side1 = (
            u.0 * su + normal.0,
            u.1 * su + normal.1,
            u.2 * su + normal.2
        );
        let side2 = (
            v.0 * sv + normal.0,
            v.1 * sv + normal.1,
            v.2 * sv + normal.2
        );
        let corner = (
            side1.0 + side2.0 - normal.0,
            side1.1 + side2.1 - normal.1,
            side1.2 + side2.2 - normal.2,
        );

        [corner, side1, side2]
    }

    /// Makes the greedy mesh for a single axis, in both directions (+, -).
    /// Axis : 0 = X, 1 = Y, Z = 2
    pub fn make_greedy_axis(padded_chunk: &PaddedChunk, vertices: &mut Vec<Vertex>, cx: i32, cy: i32, cz: i32, axis: i32) {
        let chunk_origin = Vector3::new((cx as f32) * CHUNK_SIZE_F, (cy as f32) * CHUNK_SIZE_F, (cz as f32) * CHUNK_SIZE_F);

        // Local bases
        // D is the main axis (for axis = 0, it is X)
        // U is the secondary axis (for axis = 0, it is Y)
        // V is the tertiary axis (for axis = 0, it is Z)
        let mut e_d = [0; 3];
        let mut e_u = [0; 3];
        let mut e_v = [0; 3];

        let axes = ["X", "  Y", "    Z"];
        let axis_str = axes[axis as usize];

        e_d[axis as usize] = 1;
        e_u[((axis + 1) % 3) as usize] = 1;
        e_v[((axis + 2) % 3) as usize] = 1;

        let e_d = Vector3::new(e_d[0], e_d[1], e_d[2]);
        let e_u = Vector3::new(e_u[0], e_u[1], e_u[2]);
        let e_v = Vector3::new(e_v[0], e_v[1], e_v[2]);

        let e_d_f = Vector3::new(e_d[0] as f32, e_d[1] as f32, e_d[2] as f32);
        let e_u_f = Vector3::new(e_u[0] as f32, e_u[1] as f32, e_u[2] as f32);
        let e_v_f = Vector3::new(e_v[0] as f32, e_v[1] as f32, e_v[2] as f32);

        const CLEARED_MASK: [[FaceMask; CHUNK_SIZE as usize]; CHUNK_SIZE as usize] =
            [[FaceMask::empty(); CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
        let mut mask: [[FaceMask; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

        // Faces enum's dictionary based on the axis used
        // [0]s are positive axis
        // [1]s are negative axis
        let faces: [Direction; 2] = match axis {
            0 => [Direction::Right, Direction::Left],
            1 => [Direction::Top, Direction::Bottom],
            2 => [Direction::Front, Direction::Back],
            _ => unreachable!(),
        };

        let mut solidity = [false; PADDED_CHUNK_BLOCK_NUMBER];

        for i in 0..PADDED_CHUNK_BLOCK_NUMBER {
            solidity[i] = padded_chunk.get_block_from_i(i).is_solid();
        }

        // D loop must occur CHUNK_SIZE + 1 times since for N blocs there are N + 1 possible faces (pointing out of the chunk and in between each block)
        for d in 0..=CHUNK_SIZE {
            let last_e_d = e_d * (d-1);
            time!(format!("{} reset", axis_str), {
                mask = CLEARED_MASK;
            });

            let d_f = d as f32;
            time!(format!("{} masking", axis_str), {
                // MASKING + AO
                for u in 0..=LAST_CHUNK_AXIS_INDEX {
                    let current_e_u = e_u * u;
                    for v in 0..=LAST_CHUNK_AXIS_INDEX {
                        let previous_pos = last_e_d + current_e_u + e_v * v;
                        let current_pos = previous_pos + e_d;

                        let previous_is_solid =
                            solidity[((previous_pos[0] + 1) + (previous_pos[1] + 1) * PADDED_CHUNK_SIZE + (previous_pos[2] + 1) * PADDED_CHUNK_SIZE_SQR) as usize];
                        let current_is_solid = 
                            solidity[((current_pos[0] + 1) + (current_pos[1] + 1) * PADDED_CHUNK_SIZE + (current_pos[2] + 1) * PADDED_CHUNK_SIZE_SQR) as usize];

                        if previous_is_solid == current_is_solid {
                            continue;
                        }

                        if !previous_is_solid && current_is_solid {
                            let current = padded_chunk.get_block_from_chunk_xyz(current_pos[0], current_pos[1], current_pos[2]);

                            let vertex_0_neighbors = ChunkMesh::get_ao_offsets(faces[1], Corner::BottomLeft);
                            let vertex_1_neighbors = ChunkMesh::get_ao_offsets(faces[1], Corner::BottomRight);
                            let vertex_2_neighbors = ChunkMesh::get_ao_offsets(faces[1], Corner::TopLeft);
                            let vertex_3_neighbors = ChunkMesh::get_ao_offsets(faces[1], Corner::TopRight);

                            let vertex_0_ao = ChunkMesh::get_v_ao(padded_chunk, current_pos.into(), vertex_0_neighbors);
                            let vertex_1_ao = ChunkMesh::get_v_ao(padded_chunk, current_pos.into(), vertex_1_neighbors);
                            let vertex_2_ao = ChunkMesh::get_v_ao(padded_chunk, current_pos.into(), vertex_2_neighbors);
                            let vertex_3_ao = ChunkMesh::get_v_ao(padded_chunk, current_pos.into(), vertex_3_neighbors);

                            let ao_packed = (vertex_0_ao << 6) | (vertex_1_ao << 4) | (vertex_2_ao << 2) | (vertex_3_ao << 0);

                            // We mark the mask as unvisited so the mesher will know we need to make a face out of this
                            mask[u as usize][v as usize] = FaceMask::from(
                                false,
                                current.texture_index(),
                                match axis {
                                    0 => Direction::Left,
                                    1 => Direction::Bottom,
                                    2 => Direction::Back,
                                    _ => unreachable!(),
                                },
                                ao_packed,
                            );
                        }
                        else {
                            let previous =
                                padded_chunk.get_block_from_chunk_xyz(previous_pos[0], previous_pos[1], previous_pos[2]);

                            let vertex_0_neighbors = ChunkMesh::get_ao_offsets(faces[0], Corner::BottomLeft);
                            let vertex_1_neighbors = ChunkMesh::get_ao_offsets(faces[0], Corner::BottomRight);
                            let vertex_2_neighbors = ChunkMesh::get_ao_offsets(faces[0], Corner::TopLeft);
                            let vertex_3_neighbors = ChunkMesh::get_ao_offsets(faces[0], Corner::TopRight);

                            let vertex_0_ao = ChunkMesh::get_v_ao(padded_chunk, previous_pos.into(), vertex_0_neighbors);
                            let vertex_1_ao = ChunkMesh::get_v_ao(padded_chunk, previous_pos.into(), vertex_1_neighbors);
                            let vertex_2_ao = ChunkMesh::get_v_ao(padded_chunk, previous_pos.into(), vertex_2_neighbors);
                            let vertex_3_ao = ChunkMesh::get_v_ao(padded_chunk, previous_pos.into(), vertex_3_neighbors);

                            let ao_packed = (vertex_0_ao << 6) | (vertex_1_ao << 4) | (vertex_2_ao << 2) | (vertex_3_ao << 0);

                            // We mark the mask as unvisited so the mesher will know we need to make a face out of this
                            mask[u as usize][v as usize] = FaceMask::from(
                                false,
                                previous.texture_index(),
                                match axis {
                                    0 => Direction::Right,
                                    1 => Direction::Top,
                                    2 => Direction::Front,
                                    _ => unreachable!(),
                                },
                                ao_packed,
                            );
                        }
                    }
                }
            });

            time!(format!("{} meshing", axis_str), {
                // MESHING
                for u in 0..=LAST_CHUNK_AXIS_INDEX_USIZE {
                    let mut v = 0;

                    while v <= LAST_CHUNK_AXIS_INDEX_USIZE {
                        let face = mask[u][v];

                        if face.get_visited() {
                            v += 1;
                            continue;
                        }

                        mask[u][v].set_visited(true);

                        let mut width = 1;
                        let mut height = 1;

                        // Expansion in the U axis
                        for u_2 in (u + 1)..=LAST_CHUNK_AXIS_INDEX_USIZE {
                            if width >= GREEDY_MESH_MAX_FACE_WIDTH || mask[u_2][v].get_visited() || mask[u_2][v].data != face.data {
                                break;
                            }
                            width += 1;
                            mask[u_2][v].set_visited(true);
                        }

                        // Expansion in the V axis
                        'expand: for v_2 in (v + 1)..=LAST_CHUNK_AXIS_INDEX_USIZE {
                            if height >= GREEDY_MESH_MAX_FACE_HEIGHT {
                                break;
                            }
                            // For each time we increment in the V axis, we must verify that every block in the U axis is compatible.
                            for u_2 in u..(u + width) {
                                if mask[u_2][v_2].get_visited() || mask[u_2][v_2].data != face.data {
                                    break 'expand;
                                }
                            }

                            height += 1;

                            for u_2 in u..(u + width) {
                                mask[u_2][v_2].set_visited(true);
                            }
                        }

                        let u_f32 = u as f32;
                        let v_f32 = v as f32;
                        let w_f32 = width as f32;
                        let h_f32 = height as f32;

                        let block_pos = chunk_origin + e_v_f * v_f32 + e_u_f * u_f32 + e_d_f * d_f;

                        let e_u_w = e_u_f * w_f32;
                        let e_v_h = e_v_f * h_f32;
                        let e_uv_wh = e_u_w + e_v_h;

                        let local_position_v0 = block_pos;
                        let local_position_v1 = block_pos + e_v_h;
                        let local_position_v2 = block_pos + e_u_w;
                        let local_position_v3 = block_pos + e_uv_wh;

                        let vertex_0_ao = face.get_ao() >> 6;
                        let vertex_1_ao = (face.get_ao() >> 4) & 0b11;
                        let vertex_2_ao = (face.get_ao() >> 2) & 0b11;
                        let vertex_3_ao = face.get_ao() & 0b11;

                        let texture_index = face.get_block_id();

                        let uv_u0 = 0.0;
                        let uv_v0 = 0.0;
                        let uv_u1 = w_f32;
                        let uv_v1 = h_f32;

                        let vertex_0 = Vertex::new(
                            local_position_v0[0] as f32,
                            local_position_v0[1] as f32,
                            local_position_v0[2] as f32,
                            texture_index,
                            (vertex_0_ao as i32) as f32,
                            uv_u0,
                            uv_v0,
                        );
                        let vertex_1 = Vertex::new(
                            local_position_v1[0] as f32,
                            local_position_v1[1] as f32,
                            local_position_v1[2] as f32,
                            texture_index,
                            (vertex_1_ao as i32) as f32,
                            uv_u0,
                            uv_v1,
                        );
                        let vertex_2 = Vertex::new(
                            local_position_v2[0] as f32,
                            local_position_v2[1] as f32,
                            local_position_v2[2] as f32,
                            texture_index,
                            (vertex_2_ao as i32) as f32,
                            uv_u1,
                            uv_v0,
                        );
                        let vertex_3 = Vertex::new(
                            local_position_v3[0] as f32,
                            local_position_v3[1] as f32,
                            local_position_v3[2] as f32,
                            texture_index,
                            (vertex_3_ao as i32) as f32,
                            uv_u1,
                            uv_v1,
                        );

                        let reverse_faces = face.get_face().is_negative();

                        // Because of back culling, we must invert the normal of the face by swaping vertices of the triangles on the horizontal axis
                        if reverse_faces {
                            vertices.extend_from_slice(&[vertex_0, vertex_1, vertex_2, vertex_2, vertex_1, vertex_3]);
                        } else {
                            vertices.extend_from_slice(&[vertex_1, vertex_0, vertex_3, vertex_3, vertex_0, vertex_2]);
                        }

                        v += height;
                    }
                }
            })
        }
    }

    /// Makes the greedy mesh for the x axis, in both directions (+, -).
    pub fn make_greedy_x(padded_chunk: &PaddedChunk, vertices: &mut Vec<Vertex>, cx: i32, cy: i32, cz: i32) {
        
        // Local bases
        // D is the main axis (X)
        // U is the secondary axis (Y)
        // V is the tertiary axis (Z)

        const CLEARED_MASK: [[FaceMask; CHUNK_SIZE as usize]; CHUNK_SIZE as usize] =
            [[FaceMask::empty(); CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

        const FACES: [Direction; 2] = [Direction::Right, Direction::Left];

        const NEIGHBORS: [[[(i32, i32, i32); 3]; 4]; 2] = [
            [
                ChunkMesh::get_ao_optimized_offsets(FACES[0], Corner::BottomLeft),
                ChunkMesh::get_ao_optimized_offsets(FACES[0], Corner::BottomRight),
                ChunkMesh::get_ao_optimized_offsets(FACES[0], Corner::TopLeft),
                ChunkMesh::get_ao_optimized_offsets(FACES[0], Corner::TopRight),
            ],
            [
                ChunkMesh::get_ao_optimized_offsets(FACES[1], Corner::BottomLeft),
                ChunkMesh::get_ao_optimized_offsets(FACES[1], Corner::BottomRight),
                ChunkMesh::get_ao_optimized_offsets(FACES[1], Corner::TopLeft),
                ChunkMesh::get_ao_optimized_offsets(FACES[1], Corner::TopRight),
            ],
        ];

        let chunk_origin_x = (cx as f32) * CHUNK_SIZE_F;
        let chunk_origin_y = (cy as f32) * CHUNK_SIZE_F;
        let chunk_origin_z = (cz as f32) * CHUNK_SIZE_F;

        let mut mask: [[FaceMask; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
        let mut solidity = [false; PADDED_CHUNK_BLOCK_NUMBER];

        for i in 0..PADDED_CHUNK_BLOCK_NUMBER {
            solidity[i] = padded_chunk.get_block_from_i(i).is_solid();
        }

        // D loop must occur CHUNK_SIZE + 1 times since for N blocs there are N + 1 possible faces (pointing out of the chunk and in between each block)
        for d in 0..=CHUNK_SIZE {
            time!("X reset", {
                mask = CLEARED_MASK;
            });

            let d_f = d as f32;
            let padded_d = d + 1;
            time!("X masking", {
                // MASKING + AO
                for u in 1..=LAST_PADDED_CHUNK_CENTER_INDEX {
                    let u_minus_1 = (u - 1) as usize;
                    for v in 1..=LAST_PADDED_CHUNK_CENTER_INDEX {
                        let previous_pos = [d, u, v];
                        let current_pos = [padded_d, u, v];

                        let previous_i = (
                            previous_pos[0] +
                            previous_pos[1] * PADDED_CHUNK_SIZE +
                            previous_pos[2] * PADDED_CHUNK_SIZE_SQR
                        ) as usize;
                        // let current_i = (current_pos[0] + current_pos[1] * PADDED_CHUNK_SIZE + current_pos[2] * PADDED_CHUNK_SIZE_SQR) as usize;
                        let current_i = previous_i + 1;

                        let previous_is_solid = solidity[previous_i];
                        let current_is_solid = solidity[current_i];

                        if previous_is_solid == current_is_solid {
                            continue;
                        }

                        if !previous_is_solid && current_is_solid {
                            // let current = padded_chunk.get_block_from_xyz(current_pos[0], current_pos[1], current_pos[2]);
                            let current = padded_chunk.get_block_from_i(current_i);

                            let vertex_0_ao = ChunkMesh::get_v_unpadded_ao(padded_chunk, current_pos, NEIGHBORS[1][0]);
                            let vertex_1_ao = ChunkMesh::get_v_unpadded_ao(padded_chunk, current_pos, NEIGHBORS[1][1]);
                            let vertex_2_ao = ChunkMesh::get_v_unpadded_ao(padded_chunk, current_pos, NEIGHBORS[1][2]);
                            let vertex_3_ao = ChunkMesh::get_v_unpadded_ao(padded_chunk, current_pos, NEIGHBORS[1][3]);

                            let ao_packed = (vertex_0_ao << 6) | (vertex_1_ao << 4) | (vertex_2_ao << 2) | (vertex_3_ao << 0);

                            // We mark the mask as unvisited so the mesher will know we need to make a face out of this
                            // -1 because u and v are indices for PADDED_CHUNK, but mask contains indices for a basic CHUNK
                            mask[u_minus_1][(v-1) as usize] = FaceMask::from(
                                false,
                                current.texture_index(),
                                FACES[1],
                                ao_packed,
                            );
                        }
                        else {
                            // let previous = padded_chunk.get_block_from_xyz(previous_pos[0], previous_pos[1], previous_pos[2]);
                            let previous = padded_chunk.get_block_from_i(previous_i);

                            let vertex_0_ao = ChunkMesh::get_v_unpadded_ao(padded_chunk, previous_pos, NEIGHBORS[0][0]);
                            let vertex_1_ao = ChunkMesh::get_v_unpadded_ao(padded_chunk, previous_pos, NEIGHBORS[0][1]);
                            let vertex_2_ao = ChunkMesh::get_v_unpadded_ao(padded_chunk, previous_pos, NEIGHBORS[0][2]);
                            let vertex_3_ao = ChunkMesh::get_v_unpadded_ao(padded_chunk, previous_pos, NEIGHBORS[0][3]);

                            let ao_packed = (vertex_0_ao << 6) | (vertex_1_ao << 4) | (vertex_2_ao << 2) | (vertex_3_ao << 0);

                            // We mark the mask as unvisited so the mesher will know we need to make a face out of this
                            // -1 because u and v are indices for PADDED_CHUNK, but mask contains indices for a basic CHUNK
                            mask[u_minus_1][(v-1) as usize] = FaceMask::from(
                                false,
                                previous.texture_index(),
                                FACES[0],
                                ao_packed,
                            );
                        }
                    }
                }
            });

            time!("X meshing", {
                // MESHING
                for u in 0..=LAST_CHUNK_AXIS_INDEX_USIZE {
                    let mut v = 0;

                    while v <= LAST_CHUNK_AXIS_INDEX_USIZE {
                        let face = mask[u][v];

                        if face.get_visited() {
                            v += 1;
                            continue;
                        }

                        mask[u][v].set_visited(true);

                        let mut width = 1;
                        let mut height = 1;

                        // Expansion in the U axis
                        for u_2 in (u + 1)..=LAST_CHUNK_AXIS_INDEX_USIZE {
                            if width >= GREEDY_MESH_MAX_FACE_WIDTH || mask[u_2][v].get_visited() || mask[u_2][v].data != face.data {
                                break;
                            }
                            width += 1;
                            mask[u_2][v].set_visited(true);
                        }

                        // Expansion in the V axis
                        'expand: for v_2 in (v + 1)..=LAST_CHUNK_AXIS_INDEX_USIZE {
                            if height >= GREEDY_MESH_MAX_FACE_HEIGHT {
                                break;
                            }
                            // For each time we increment in the V axis, we must verify that every block in the U axis is compatible.
                            for u_2 in u..(u + width) {
                                if mask[u_2][v_2].get_visited() || mask[u_2][v_2].data != face.data {
                                    break 'expand;
                                }
                            }

                            height += 1;

                            for u_2 in u..(u + width) {
                                mask[u_2][v_2].set_visited(true);
                            }
                        }

                        let u_f32 = u as f32;
                        let v_f32 = v as f32;
                        let w_f32 = width as f32;
                        let h_f32 = height as f32;

                        let block_pos_x = chunk_origin_x + d_f;
                        let block_pos_y = chunk_origin_y + u_f32;
                        let block_pos_z = chunk_origin_z + v_f32;

                        let block_pos_u = block_pos_y + w_f32;
                        let block_pos_v = block_pos_z + h_f32;

                        let v0_pos = [block_pos_x, block_pos_y, block_pos_z];
                        let v1_pos = [block_pos_x, block_pos_y, block_pos_v];
                        let v2_pos = [block_pos_x, block_pos_u, block_pos_z];
                        let v3_pos = [block_pos_x, block_pos_u, block_pos_v];

                        let v0_ao = face.get_ao() >> 6;
                        let v1_ao = (face.get_ao() >> 4) & 0b11;
                        let v2_ao = (face.get_ao() >> 2) & 0b11;
                        let v3_ao = face.get_ao() & 0b11;

                        let texture_index = face.get_block_id();

                        let uv_u0 = 0.0;
                        let uv_v0 = 0.0;
                        let uv_u1 = w_f32;
                        let uv_v1 = h_f32;

                        let vertex_0 = Vertex::new(
                            v0_pos[0],
                            v0_pos[1],
                            v0_pos[2],
                            texture_index,
                            (v0_ao as i32) as f32,
                            uv_u0,
                            uv_v0,
                        );
                        let vertex_1 = Vertex::new(
                            v1_pos[0],
                            v1_pos[1],
                            v1_pos[2],
                            texture_index,
                            (v1_ao as i32) as f32,
                            uv_u0,
                            uv_v1,
                        );
                        let vertex_2 = Vertex::new(
                            v2_pos[0],
                            v2_pos[1],
                            v2_pos[2],
                            texture_index,
                            (v2_ao as i32) as f32,
                            uv_u1,
                            uv_v0,
                        );
                        let vertex_3 = Vertex::new(
                            v3_pos[0],
                            v3_pos[1],
                            v3_pos[2],
                            texture_index,
                            (v3_ao as i32) as f32,
                            uv_u1,
                            uv_v1,
                        );

                        let reverse_faces = face.get_face().is_negative();

                        // Because of back culling, we must invert the normal of the face by swaping vertices of the triangles on the horizontal axis
                        if reverse_faces {
                            vertices.extend_from_slice(&[vertex_0, vertex_1, vertex_2, vertex_2, vertex_1, vertex_3]);
                        } else {
                            vertices.extend_from_slice(&[vertex_1, vertex_0, vertex_3, vertex_3, vertex_0, vertex_2]);
                        }

                        v += height;
                    }
                }
            })
        }
    }

    pub fn update(&mut self, vertices: Vec<Vertex>, renderer: &mut Renderer) {
        self.dirty.store(false, Ordering::Relaxed);

        if let Some(mesh_id) = self.id {
            if vertices.len() == 0 {
                renderer.render_manager.mesh_manager.free_data(mesh_id);
                self.id = None;
            } else {
                renderer.render_manager.mesh_manager.update_data(
                    &renderer.gpu_context.device,
                    &renderer.gpu_context.queue,
                    renderer.frame_encoder.as_mut().unwrap(),
                    DataEntry::new(mesh_id, bytemuck::cast_slice(&vertices)),
                );
            }
        } else {
            if vertices.len() == 0 {
                return;
            }
            self.id = Some(
                renderer
                    .render_manager
                    .mesh_manager
                    .add_data(
                        &renderer.gpu_context.device,
                        &renderer.gpu_context.queue,
                        renderer.frame_encoder.as_mut().unwrap(),
                        bytemuck::cast_slice(&vertices),
                    )
                    .expect(&format!("Could not add data - data len: {}", &vertices.len())),
            );
        }
    }
}

pub struct GreedyMeshingProcessor;

impl Parallelizable for GreedyMeshingProcessor {
    type Context = ();
    type Input = (Arc<Chunk>, MeshSnapshot, i32, i32, i32);
    type Output = Option<Vec<Vertex>>;

    // Make greedy
    fn process(input: Self::Input, _ctx: &Self::Context) -> Self::Output {
        let (main_chunk, neighbors, cx, cy, cz) = input;

        let padded = PaddedChunk::from_snapshot(&main_chunk, &neighbors);
        let mut vertices = Vec::new();

        time!("greedy_meshing", {
            ChunkMesh::make_greedy_x(&padded, &mut vertices, cx, cy, cz);
            ChunkMesh::make_greedy_axis(&padded, &mut vertices, cx, cy, cz, 1);
            ChunkMesh::make_greedy_axis(&padded, &mut vertices, cx, cy, cz, 2);
        });

        return Some(vertices);
    }
}
