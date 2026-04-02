use std::sync::atomic::{AtomicBool, Ordering};

use cgmath::Vector3;

use crate::{
    common::geometry::{direction::Direction, vertex::Vertex},
    engine::render::{
        mesh::face_mask::FaceMask,
        render::{MeshData, MeshId, Renderer},
    },
    game::world::{
        chunk::{Chunk, CHUNK_SIZE, LAST_CHUNK_AXIS_INDEX, LAST_CHUNK_AXIS_INDEX_USIZE},
        padded_chunk::PaddedChunk,
        world::World,
    },
};

enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub struct ChunkMesh {
    pub id: Option<MeshId>,
    dirty: AtomicBool,
}

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

    pub fn get_v_ao(chunk: &PaddedChunk, pos: Vector3<i32>, neighbors: [(i32, i32, i32); 3]) -> u8 {
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

        let side1 = (u.0 * su + normal.0, u.1 * su + normal.1, u.2 * su + normal.2);
        let side2 = (v.0 * sv + normal.0, v.1 * sv + normal.1, v.2 * sv + normal.2);
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
        // if axis != 1 {
        //     return;
        // }
       
        let chunk_origin = Vector3::new(cx * CHUNK_SIZE, cy * CHUNK_SIZE, cz * CHUNK_SIZE);

        // Local bases
        // D is the main axis (for axis = 0, it is X)
        // U is the secondary axis (for axis = 0, it is Y)
        // V is the tertiary axis (for axis = 0, it is Z)
        let mut e_d = [0; 3];
        let mut e_u = [0; 3];
        let mut e_v = [0; 3];

        e_d[axis as usize] = 1;
        e_u[((axis + 1) % 3) as usize] = 1;
        e_v[((axis + 2) % 3) as usize] = 1;

        let e_d = Vector3::new(e_d[0], e_d[1], e_d[2]);
        let e_u = Vector3::new(e_u[0], e_u[1], e_u[2]);
        let e_v = Vector3::new(e_v[0], e_v[1], e_v[2]);

        let mut mask: [[FaceMask; CHUNK_SIZE as usize]; CHUNK_SIZE as usize] =
            [[FaceMask::empty(); CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

        // Faces enum's dictionary based on the axis used
        // [0]s are positive axis
        // [1]s are negative axis
        let faces: [Direction; 2] = match axis {
            0 => [Direction::Right, Direction::Left],
            1 => [Direction::Top, Direction::Bottom],
            2 => [Direction::Front, Direction::Back],
            _ => unreachable!(),
        };

        // D loop must occur CHUNK_SIZE + 1 times since for N blocs there are N + 1 possible faces (pointing out of the chunk and in between each block)
        for d in 0..=CHUNK_SIZE {
            
            // MASKING + AO
            for u in 0..=LAST_CHUNK_AXIS_INDEX {
                for v in 0..=LAST_CHUNK_AXIS_INDEX {
                    let previous_pos = e_d * (d - 1) + e_u * u + e_v * v;
                    let current_pos = e_d * d + e_u * u + e_v * v;

                    let previous = padded_chunk.get_block_from_chunk_xyz(previous_pos[0], previous_pos[1], previous_pos[2]);
                    let current = padded_chunk.get_block_from_chunk_xyz(current_pos[0], current_pos[1], current_pos[2]);

                    match (previous.is_solid(), current.is_solid()) {
                        // If both blocks are either plain or air, making faces is useless
                        (true, true) | (false, false) => {}
                        (false, true) => {
                            let vertex_0_neighbors = ChunkMesh::get_ao_offsets(faces[1], Corner::BottomLeft);
                            let vertex_1_neighbors = ChunkMesh::get_ao_offsets(faces[1], Corner::BottomRight);
                            let vertex_2_neighbors = ChunkMesh::get_ao_offsets(faces[1], Corner::TopLeft);
                            let vertex_3_neighbors = ChunkMesh::get_ao_offsets(faces[1], Corner::TopRight);

                            let vertex_0_ao = ChunkMesh::get_v_ao(padded_chunk, current_pos, vertex_0_neighbors);
                            let vertex_1_ao = ChunkMesh::get_v_ao(padded_chunk, current_pos, vertex_1_neighbors);
                            let vertex_2_ao = ChunkMesh::get_v_ao(padded_chunk, current_pos, vertex_2_neighbors);
                            let vertex_3_ao = ChunkMesh::get_v_ao(padded_chunk, current_pos, vertex_3_neighbors);

                            let ao_packed = (vertex_0_ao << 6) | (vertex_1_ao << 4) | (vertex_2_ao << 2) | (vertex_3_ao << 0);

                            // We mark the mask as unvisited so the mesher will know we need to make a face out of this
                            mask[u as usize][v as usize] = FaceMask::from(
                                false,
                                current.id,
                                match axis {
                                    0 => Direction::Left,
                                    1 => Direction::Bottom,
                                    2 => Direction::Back,
                                    _ => unreachable!(),
                                },
                                ao_packed,
                            );

                            // println!("1 visited: {}", mask[u as usize][v as usize].get_visited());
                        }
                        (true, false) => {
                            let vertex_0_neighbors = ChunkMesh::get_ao_offsets(faces[0], Corner::BottomLeft);
                            let vertex_1_neighbors = ChunkMesh::get_ao_offsets(faces[0], Corner::BottomRight);
                            let vertex_2_neighbors = ChunkMesh::get_ao_offsets(faces[0], Corner::TopLeft);
                            let vertex_3_neighbors = ChunkMesh::get_ao_offsets(faces[0], Corner::TopRight);

                            let vertex_0_ao = ChunkMesh::get_v_ao(padded_chunk, previous_pos, vertex_0_neighbors);
                            let vertex_1_ao = ChunkMesh::get_v_ao(padded_chunk, previous_pos, vertex_1_neighbors);
                            let vertex_2_ao = ChunkMesh::get_v_ao(padded_chunk, previous_pos, vertex_2_neighbors);
                            let vertex_3_ao = ChunkMesh::get_v_ao(padded_chunk, previous_pos, vertex_3_neighbors);

                            let ao_packed = (vertex_0_ao << 6) | (vertex_1_ao << 4) | (vertex_2_ao << 2) | (vertex_3_ao << 0);

                            // We mark the mask as unvisited so the mesher will know we need to make a face out of this
                            mask[u as usize][v as usize] = FaceMask::from(
                                false,
                                previous.id,
                                match axis {
                                    0 => Direction::Right,
                                    1 => Direction::Top,
                                    2 => Direction::Front,
                                    _ => unreachable!(),
                                },
                                ao_packed,
                            );

                            // println!("2 visited: {}", mask[u as usize][v as usize].get_visited());
                        }
                    }
                }
            }

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
                    for u_2 in u..=LAST_CHUNK_AXIS_INDEX_USIZE {
                        if mask[u_2][v].get_visited() || !mask[u_2][v].can_merge_with(&face) {
                            break;
                        }
                        width += 1;
                        mask[u_2][v].set_visited(true);
                    }

                    // Expansion in the V axis
                    'expand: for v_2 in v..=LAST_CHUNK_AXIS_INDEX_USIZE {
                        // For each time we increment in the V axis, we must verify that every block in the U axis is compatible. If not, we stop the expansion.
                        for u_2 in u..(u + width) {
                            if mask[u_2][v_2].get_visited() || !mask[u_2][v_2].can_merge_with(&face) {
                                break 'expand;
                            }
                        }

                        height += 1;

                        for iu in u..(u + width) {
                            mask[iu][v_2].set_visited(true);
                        }
                    }

                    let u_i32 = u as i32;
                    let v_i32 = v as i32;
                    let w_i32 = width as i32;
                    let h_i32 = height as i32;

                    let block_pos = chunk_origin + e_v * v_i32 + e_u * u_i32 + e_d * d;

                    let e_u_w = e_u * w_i32;
                    let e_v_h = e_v * h_i32;
                    let e_uv_wh = e_u_w + e_v_h;

                    let local_position_v0 = block_pos;
                    let local_position_v1 = block_pos + e_v_h;
                    let local_position_v2 = block_pos + e_u_w;
                    let local_position_v3 = block_pos + e_uv_wh;

                    // Bottom left
                    // Bottom right
                    // Top left
                    // Top right
                    let vertex_0_ao = face.get_ao() >> 6;
                    let vertex_1_ao = (face.get_ao() >> 4) & 0b11;
                    let vertex_2_ao = (face.get_ao() >> 2) & 0b11;
                    let vertex_3_ao = face.get_ao() & 0b11;

                    let v0 = Vertex::new(
                        local_position_v0[0] as f32,
                        local_position_v0[1] as f32,
                        local_position_v0[2] as f32,
                        0,
                        (vertex_0_ao as i32) as f32,
                    );
                    let v1 = Vertex::new(
                        local_position_v1[0] as f32,
                        local_position_v1[1] as f32,
                        local_position_v1[2] as f32,
                        0,
                        (vertex_1_ao as i32) as f32,
                    );
                    let v2 = Vertex::new(
                        local_position_v2[0] as f32,
                        local_position_v2[1] as f32,
                        local_position_v2[2] as f32,
                        0,
                        (vertex_2_ao as i32) as f32,
                    );
                    let v3 = Vertex::new(
                        local_position_v3[0] as f32,
                        local_position_v3[1] as f32,
                        local_position_v3[2] as f32,
                        0,
                        (vertex_3_ao as i32) as f32,
                    );

                    // Because of back culling, we must invert the normal of the face by swaping vertices of the triangles on the horizontal axis
                    let reverse_faces = face.get_face().is_negative();

                    if reverse_faces {
                        vertices.extend_from_slice(&[v0, v1, v2, v2, v1, v3]);
                    }
                    else {
                        vertices.extend_from_slice(&[v1, v0, v3, v3, v0, v2]);
                    }

                    v += height;
                }
            }
        }
    }

    /// Makes the greedy mesh of a chunk.
    /// Very expensive operation.
    /// Should not be called each frame/frequently.
    /// Limit usage to necessary.
    pub fn make_greedy(&mut self, chunk: &Chunk, world: &World, renderer: &mut Renderer, cx: i32, cy: i32, cz: i32) {
        let mut vertices: Vec<Vertex> = vec![];
        let padded_chunk = PaddedChunk::new(chunk, world);

        ChunkMesh::make_greedy_axis(&padded_chunk, &mut vertices, cx, cy, cz, 0);
        ChunkMesh::make_greedy_axis(&padded_chunk, &mut vertices, cx, cy, cz, 1);
        ChunkMesh::make_greedy_axis(&padded_chunk, &mut vertices, cx, cy, cz, 2);

        self.dirty.store(false, Ordering::Relaxed);

        if let Some(mesh_id) = self.id {
            renderer.render_manager.update_mesh(
                &renderer.gpu_context.device,
                &renderer.gpu_context.queue,
                MeshData::new(vertices, None),
                mesh_id,
            );
        } else {
            self.id = Some(renderer.render_manager.allocate_mesh(
                &renderer.gpu_context.device,
                &renderer.gpu_context.queue,
                MeshData::new(vertices, None),
            ));
        }
    }
}
