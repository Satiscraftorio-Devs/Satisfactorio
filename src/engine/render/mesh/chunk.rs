use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    common::geometry::{direction::Direction, vertex::Vertex},
    engine::render::{
        mesh::face_mask::FaceMask, render::{MeshData, MeshId, Renderer},
    },
    game::world::{
        block::BlockInstance,
        chunk::{CHUNK_SIZE, CHUNK_USIZE, Chunk, LAST_CHUNK_AXIS_INDEX, LAST_CHUNK_AXIS_INDEX_USIZE},
        padded_chunk::{self, LAST_PADDED_CHUNK_AXIS_INDEX, PADDED_CHUNK_SIZE, PaddedChunk},
        world::World,
    },
};

pub struct ChunkMesh {
    pub mesh_id: Option<MeshId>,
    dirty: AtomicBool,
}

impl ChunkMesh {
    pub fn new() -> ChunkMesh {
        return ChunkMesh {
            mesh_id: None, // Not yet created
            dirty: AtomicBool::new(true),
        };
    }

    pub fn set_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    pub fn is_dirty(&self) -> bool {
        return self.dirty.load(Ordering::Relaxed);
    }

    pub fn get_v_ao(
        chunk: &PaddedChunk,
        pos: [i32; 3],
        neighbors: [[i32; 3]; 3],
    ) -> u8 {
        let corner_solid = chunk.get_block_from_chunk_xyz(pos[0] + neighbors[0][0], pos[1] + neighbors[0][1], pos[2] + neighbors[0][2]).is_solid() as u8;
        let side1_solid = chunk.get_block_from_chunk_xyz(pos[0] + neighbors[1][0], pos[1] + neighbors[1][1], pos[2] + neighbors[1][2]).is_solid() as u8;
        let side2_solid = chunk.get_block_from_chunk_xyz(pos[0] + neighbors[2][0], pos[1] + neighbors[2][1], pos[2] + neighbors[2][2]).is_solid() as u8;

        ChunkMesh::AO_TABLE[(side1_solid | (side2_solid << 1) | (corner_solid << 2)) as usize]
    }

    const AO_TABLE: [u8; 8] = [
        3, 2, 2, 0,
        2, 1, 1, 0,
    ];

    pub fn make_greedy_axis(
        padded_chunk: &PaddedChunk,
        vertices: &mut Vec<Vertex>,
        cx: i32,
        cy: i32,
        cz: i32,
        axis: i32,
    ) {
        let base = [
            cx * CHUNK_SIZE,
            cy * CHUNK_SIZE,
            cz * CHUNK_SIZE,
        ];

        // Base locale
        let mut e_d = [0; 3];
        let mut e_u = [0; 3];
        let mut e_v = [0; 3];

        e_d[axis as usize] = 1;
        e_u[((axis + 1) % 3) as usize] = 1;
        e_v[((axis + 2) % 3) as usize] = 1;

        let mut mask: [[FaceMask; CHUNK_SIZE as usize]; CHUNK_SIZE as usize] =
            [[FaceMask::empty(); CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

        // [AXIS][DIRECTION][VERTEX][COORDINATE]
        const vertex_positions: [[[[i32; 3]; 4]; 2]; 3] = [
            // X axis
            [
                // Direction +
                [
                    [0, 0, 1],
                    [0, 0, 0],
                    [0, 1, 1],
                    [0, 1, 0],
                ],
                // Direction -
                [
                    [1, 0, 0],
                    [1, 0, 1],
                    [1, 1, 0],
                    [1, 1, 1],
                ]
            ],
            // Y axis
            [
                // Direction +
                [
                    [0, 0, 1],
                    [1, 0, 1],
                    [0, 0, 0],
                    [1, 0, 0],
                ],
                // Direction -
                [
                    [1, 1, 1],
                    [0, 1, 1],
                    [1, 1, 0],
                    [0, 1, 0],
                ]
            ],
            // Z axis
            [
                // Direction +
                [
                    [0, 0, 0],
                    [1, 0, 0],
                    [0, 1, 0],
                    [1, 1, 0],
                ],
                // Direction -
                [
                    [1, 0, 1],
                    [0, 0, 1],
                    [1, 1, 1],
                    [0, 1, 1],
                ]
            ],
        ];

        // [AXIS][DIRECTION][VERTEX][NEIGHBOR][COORDINATE]
        const neighbor_delta_positions: [[[[[i32; 3]; 3]; 4]; 2]; 3] = [
            // X axis
            [
                // +
                [
                    // Bottom left
                    [
                        // Corner
                        [0, -1, 1],
                        // Side 1
                        [0, -1, 0],
                        // Side 2
                        [0, 0, 1]
                    ],
                    // Bottom right
                    [
                        // Corner
                        [0, -1, -1],
                        // Side 1
                        [0, 0, -1],
                        // Side 2
                        [0, -1, 0]
                    ],
                    // Top left
                    [
                        // Corner
                        [0, 1, 1],
                        // Side 1
                        [0, 0, 1],
                        // Side 2
                        [0, 1, 0]
                    ],
                    // Top right
                    [
                        // Corner
                        [0, 1, -1],
                        // Side 1
                        [0, 1, 0],
                        // Side 2
                        [0, 0, -1]
                    ],
                ],
                // -
                [
                    // Bottom left
                    [
                        // Corner
                        [0, -1, -1],
                        // Side 1
                        [0, -1, 0],
                        // Side 2
                        [0, 0, -1]
                    ],
                    // Bottom right
                    [
                        // Corner
                        [0, -1, 1],
                        // Side 1
                        [0, 0, 1],
                        // Side 2
                        [0, -1, 0]
                    ],
                    // Top left
                    [
                        // Corner
                        [0, 1, -1],
                        // Side 1
                        [0, 0, -1],
                        // Side 2
                        [0, 1, 0]
                    ],
                    // Top right
                    [
                        // Corner
                        [0, 1, 1],
                        // Side 1
                        [0, 1, 0],
                        // Side 2
                        [0, 0, 1]
                    ],
                ]
            ],
            // Y axis
            [
                // +
                [
                    // Bottom left
                    [
                        // Corner
                        [0, 0, 0],
                        // Side 1
                        [0, 0, 0],
                        // Side 2
                        [0, 0, 0]
                    ],
                    // Bottom right
                    [
                        // Corner
                        [0, 0, 0],
                        // Side 1
                        [0, 0, 0],
                        // Side 2
                        [0, 0, 0]
                    ],
                    // Top left
                    [
                        // Corner
                        [0, 0, 0],
                        // Side 1
                        [0, 0, 0],
                        // Side 2
                        [0, 0, 0]
                    ],
                    // Top right
                    [
                        // Corner
                        [0, 0, 0],
                        // Side 1
                        [0, 0, 0],
                        // Side 2
                        [0, 0, 0]
                    ],
                ],
                // -
                [
                    // Bottom left
                    [
                        // Corner
                        [0, 0, 0],
                        // Side 1
                        [0, 0, 0],
                        // Side 2
                        [0, 0, 0]
                    ],
                    // Bottom right
                    [
                        // Corner
                        [0, 0, 0],
                        // Side 1
                        [0, 0, 0],
                        // Side 2
                        [0, 0, 0]
                    ],
                    // Top left
                    [
                        // Corner
                        [0, 0, 0],
                        // Side 1
                        [0, 0, 0],
                        // Side 2
                        [0, 0, 0]
                    ],
                    // Top right
                    [
                        // Corner
                        [0, 0, 0],
                        // Side 1
                        [0, 0, 0],
                        // Side 2
                        [0, 0, 0]
                    ],
                ]
            ],
            // Z axis
            [
                // +
                [
                    // Bottom left
                    [
                        // Corner
                        [0, 0, 0],
                        // Side 1
                        [0, 0, 0],
                        // Side 2
                        [0, 0, 0]
                    ],
                    // Bottom right
                    [
                        // Corner
                        [0, 0, 0],
                        // Side 1
                        [0, 0, 0],
                        // Side 2
                        [0, 0, 0]
                    ],
                    // Top left
                    [
                        // Corner
                        [0, 0, 0],
                        // Side 1
                        [0, 0, 0],
                        // Side 2
                        [0, 0, 0]
                    ],
                    // Top right
                    [
                        // Corner
                        [0, 0, 0],
                        // Side 1
                        [0, 0, 0],
                        // Side 2
                        [0, 0, 0]
                    ],
                ],
                // -
                [
                    // Bottom left
                    [
                        // Corner
                        [0, 0, 0],
                        // Side 1
                        [0, 0, 0],
                        // Side 2
                        [0, 0, 0]
                    ],
                    // Bottom right
                    [
                        // Corner
                        [0, 0, 0],
                        // Side 1
                        [0, 0, 0],
                        // Side 2
                        [0, 0, 0]
                    ],
                    // Top left
                    [
                        // Corner
                        [0, 0, 0],
                        // Side 1
                        [0, 0, 0],
                        // Side 2
                        [0, 0, 0]
                    ],
                    // Top right
                    [
                        // Corner
                        [0, 0, 0],
                        // Side 1
                        [0, 0, 0],
                        // Side 2
                        [0, 0, 0]
                    ],
                ]
            ],
        ];

        // [DIRECTION][VERTEX][COORDINATE]
        let axis_based_face_vertex_positions = vertex_positions[axis as usize];
        // [DIRECTION][VERTEX][NEIGHBOR][COORDINATE]
        let axis_based_neighbor_delta_positions = neighbor_delta_positions[axis as usize];

        for d in 0..=LAST_CHUNK_AXIS_INDEX {
            // === MASK BUILD ===
            for u in 0..=LAST_CHUNK_AXIS_INDEX {
                for v in 0..=LAST_CHUNK_AXIS_INDEX {
                    let pos = add_i32_vec3(mul_i32_vec3(e_d, d), add_i32_vec3(mul_i32_vec3(e_u, u), mul_i32_vec3(e_v, v)));
                    let pos_next = add_i32_vec3(mul_i32_vec3(e_d, d + 1), add_i32_vec3(mul_i32_vec3(e_u, u), mul_i32_vec3(e_v, v)));

                    // println!("d: {} d+1: {}", d, d+1);

                    let previous = padded_chunk.get_block_from_chunk_xyz(pos[0], pos[1], pos[2]);
                    let current = padded_chunk.get_block_from_chunk_xyz(pos_next[0], pos_next[1], pos_next[2]);

                    match (previous.is_solid(), current.is_solid()) {
                        (true, true) | (false, false) => {}
                        (false, true) => {
                            // +
                            let vertex_0_position = axis_based_face_vertex_positions[0][0];
                            let vertex_1_position = axis_based_face_vertex_positions[0][1];
                            let vertex_2_position = axis_based_face_vertex_positions[0][2];
                            let vertex_3_position = axis_based_face_vertex_positions[0][3];

                            let vertex_0_neighbors = axis_based_neighbor_delta_positions[0][0];
                            let vertex_1_neighbors = axis_based_neighbor_delta_positions[0][1];
                            let vertex_2_neighbors = axis_based_neighbor_delta_positions[0][2];
                            let vertex_3_neighbors = axis_based_neighbor_delta_positions[0][3];

                            let vertex_0_ao = ChunkMesh::get_v_ao(padded_chunk, pos, vertex_0_neighbors);
                            let vertex_1_ao = ChunkMesh::get_v_ao(padded_chunk, pos, vertex_1_neighbors);
                            let vertex_2_ao = ChunkMesh::get_v_ao(padded_chunk, pos, vertex_2_neighbors);
                            let vertex_3_ao = ChunkMesh::get_v_ao(padded_chunk, pos, vertex_3_neighbors);

                            let ao_packed = (vertex_0_ao << 6) | (vertex_1_ao << 4) | (vertex_2_ao << 2) | (vertex_3_ao << 0);

                            mask[u as usize][v as usize] =
                                FaceMask::from(false, current.id, Direction::Left, ao_packed);
                        }
                        (true, false) => {
                            // -
                            let vertex_0_position = axis_based_face_vertex_positions[1][0];
                            let vertex_1_position = axis_based_face_vertex_positions[1][1];
                            let vertex_2_position = axis_based_face_vertex_positions[1][2];
                            let vertex_3_position = axis_based_face_vertex_positions[1][3];

                            let vertex_0_neighbors = axis_based_neighbor_delta_positions[1][0];
                            let vertex_1_neighbors = axis_based_neighbor_delta_positions[1][1];
                            let vertex_2_neighbors = axis_based_neighbor_delta_positions[1][2];
                            let vertex_3_neighbors = axis_based_neighbor_delta_positions[1][3];

                            let vertex_0_ao = ChunkMesh::get_v_ao(padded_chunk, pos, vertex_0_neighbors);
                            let vertex_1_ao = ChunkMesh::get_v_ao(padded_chunk, pos, vertex_1_neighbors);
                            let vertex_2_ao = ChunkMesh::get_v_ao(padded_chunk, pos, vertex_2_neighbors);
                            let vertex_3_ao = ChunkMesh::get_v_ao(padded_chunk, pos, vertex_3_neighbors);

                            let ao_packed = (vertex_0_ao << 6) | (vertex_1_ao << 4) | (vertex_2_ao << 2) | (vertex_3_ao << 0);

                            mask[u as usize][v as usize] =
                                FaceMask::from(false, previous.id, Direction::Right, ao_packed);
                        }
                    }
                }
            }

            // === GREEDY ===
            for u in 1..=LAST_CHUNK_AXIS_INDEX_USIZE {
                let mut v = 1;

                while v <= LAST_CHUNK_AXIS_INDEX_USIZE {
                    let face = mask[u][v];

                    if face.get_visited() {
                        v += 1;
                        continue;
                    }

                    mask[u][v].set_visited(true);

                    let mut width = 1;
                    let mut height = 1;

                    // expand U
                    for iu in (u + 1)..=LAST_CHUNK_AXIS_INDEX_USIZE {
                        if mask[iu][v].get_visited() || mask[iu][v].data != face.data {
                            break;
                        }
                        width += 1;
                        mask[iu][v].set_visited(true);
                    }

                    // expand V
                    'expand: for iv in (v + 1)..=LAST_CHUNK_AXIS_INDEX_USIZE {
                        for iu in u..(u + width) {
                            if mask[iu][iv].get_visited() || mask[iu][iv].data != face.data {
                                break 'expand;
                            }
                        }

                        height += 1;

                        for iu in u..(u + width) {
                            mask[iu][iv].set_visited(true);
                        }
                    }

                    let u_i32 = u as i32;
                    let v_i32 = v as i32;
                    let w_i32 = width as i32;
                    let h_i32 = height as i32;

                    // === POSITIONS ===
                    let local_position_v0 = axis_based_face_vertex_positions[face.get_face().is_negative() as usize][0];
                    let local_position_v1 = add_i32_vec3(axis_based_face_vertex_positions[face.get_face().is_negative() as usize][1], mul_i32_vec3(e_u, w_i32));
                    let local_position_v2 = add_i32_vec3(axis_based_face_vertex_positions[face.get_face().is_negative() as usize][2], mul_i32_vec3(e_v, h_i32));
                    let local_position_v3 = add_i32_vec3(axis_based_face_vertex_positions[face.get_face().is_negative() as usize][3], add_i32_vec3(mul_i32_vec3(e_u, w_i32), mul_i32_vec3(e_v, h_i32)));
                    // let p0 = ChunkMesh::build_pos(base, e_d, e_u, e_v, d, u_i32, v_i32);
                    // let p1 = ChunkMesh::build_pos(base, e_d, e_u, e_v, d, u_i32 + w_i32, v_i32);
                    // let p2 = ChunkMesh::build_pos(base, e_d, e_u, e_v, d, u_i32 + w_i32, v_i32 + h_i32);
                    // let p3 = ChunkMesh::build_pos(base, e_d, e_u, e_v, d, u_i32, v_i32 + h_i32);

                    let vertex_0_ao = face.get_ao() >> 6;
                    let vertex_1_ao = (face.get_ao() >> 4) & 0b11;
                    let vertex_2_ao = (face.get_ao() >> 2) & 0b11;
                    let vertex_3_ao = (face.get_ao() >> 0) & 0b11;

                    let v1 = Vertex::new(local_position_v0[0] as f32, local_position_v0[1] as f32, local_position_v0[2] as f32, 0, vertex_0_ao as i32);
                    let v2 = Vertex::new(local_position_v1[0] as f32, local_position_v1[1] as f32, local_position_v1[2] as f32, 0, vertex_1_ao as i32);
                    let v3 = Vertex::new(local_position_v2[0] as f32, local_position_v2[1] as f32, local_position_v2[2] as f32, 0, vertex_2_ao as i32);
                    let v4 = Vertex::new(local_position_v3[0] as f32, local_position_v3[1] as f32, local_position_v3[2] as f32, 0, vertex_3_ao as i32);

                    let flip = (vertex_0_ao + vertex_3_ao) > (vertex_1_ao + vertex_2_ao);

                    if !flip {
                        vertices.extend_from_slice(&[v1, v2, v3, v1, v4, v2]);
                    } else {
                        vertices.extend_from_slice(&[v1, v3, v2, v1, v2, v4]);
                    }

                    v += height;
                }
            }
        }
    }

    pub fn make_greedy(
        &mut self,
        chunk: &Chunk,
        world: &World,
        renderer: &mut Renderer,
        cx: i32,
        cy: i32,
        cz: i32,
    ) {
        let mut vertices: Vec<Vertex> = vec![];
        let padded_chunk = PaddedChunk::new(chunk, world);

        ChunkMesh::make_greedy_axis(&padded_chunk, &mut vertices, cx, cy, cz, 0);
        ChunkMesh::make_greedy_axis(&padded_chunk, &mut vertices, cx, cy, cz, 1);
        ChunkMesh::make_greedy_axis(&padded_chunk, &mut vertices, cx, cy, cz, 2);

        // let offset_x = cx * CHUNK_SIZE;
        // let offset_y = cy * CHUNK_SIZE;
        // let offset_z = cz * CHUNK_SIZE;

        // // We allocate once to avoid memory reallocation/destruction.
        // let mut mask: [[FaceMask; CHUNK_SIZE as usize]; CHUNK_SIZE as usize] =
        //     [[FaceMask::empty(); CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

        // let mut previous: BlockInstance;
        // let mut current: BlockInstance;


        // // X axis
        // for x in 0..=LAST_PADDED_CHUNK_AXIS_INDEX {
        //     // Mask
        //     for y in 1..=LAST_CHUNK_AXIS_INDEX {
        //         for z in 1..=LAST_CHUNK_AXIS_INDEX {
        //             previous = padded_chunk.get_block_from_xyz(x, y, z);
        //             current = padded_chunk.get_block_from_xyz(x + 1, y, z);

        //             match (previous.is_air(), current.is_air()) {
        //                 (true, true) | (false, false) => {
        //                     continue;
        //                 }
        //                 (true, false) => {
        //                     mask[y as usize][z as usize] =
        //                         FaceMask::from(false, current.id, Direction::Left);
        //                 }
        //                 (false, true) => {
        //                     mask[y as usize][z as usize] =
        //                         FaceMask::from(false, previous.id, Direction::Right);
        //                 }
        //             }
        //         }
        //     }

        //     // Mesh quads
        //     for y in 1..=LAST_CHUNK_AXIS_INDEX_USIZE {
        //         let mut z = 1;
        //         while z <= LAST_CHUNK_AXIS_INDEX_USIZE {
        //             let face = mask[y][z];
        //             if face.get_visited() {
        //                 z += 1;
        //                 continue;
        //             }

        //             mask[y][z].set_visited(true);

        //             let mut quad_y = 1;
        //             let mut quad_z = 1;

        //             // We grow the quad in the y-axis
        //             for iy in (y + 1)..=LAST_CHUNK_AXIS_INDEX_USIZE {
        //                 if mask[iy][z].get_visited()
        //                 || mask[iy][z].data != face.data
        //                 {
        //                     break;
        //                 }
        //                 quad_y += 1;
        //                 // Clear from the mask
        //                 mask[iy][z].set_visited(true);
        //             }

        //             // We grow the quad in the z-axis
        //             'expansion: for iz in (z + 1)..=LAST_CHUNK_AXIS_INDEX_USIZE {
        //                 // We check if every face in the y is compatible with our expansion, and if not, we stop it
        //                 for iy in y..(y + quad_y) {
        //                     if mask[iy][iz].get_visited()
        //                     || mask[iy][iz].data != face.data
        //                     {
        //                         break 'expansion;
        //                     }
        //                 }
        //                 quad_z += 1;
        //                 // Clear this space from the mask since we expand
        //                 for iy in y..(y + quad_y) {
        //                     mask[iy][iz as usize].set_visited(true);
        //                 }
        //             }

        //             // Add the quad to the mesh
        //             let is_left_face = face.get_face() == Direction::Left;


        //             let x0 = (x + offset_x) as f32;
        //             let y0 = (y + offset_y) as f32;
        //             let y1 = (y + quad_y + offset_y) as f32;
        //             let z0 = (z + offset_z) as f32;
        //             let z1 = (z + quad_z + offset_z) as f32;

        //             // directions pour face X
        //             let (dx, dy, dz) = (0, 1, 0);
        //             let (ux, uy, uz) = (0, 0, 1);

        //             // 4 coins
        //             let ao0 = ChunkMesh::get_vertex_ao(&padded_chunk, x, y, z, -dx, -dy, -dz, -ux, -uy, -uz);
        //             let ao1 = ChunkMesh::get_vertex_ao(&padded_chunk, x, y + quad_y, z, -dx, -dy, -dz, ux, uy, uz);
        //             let ao2 = ChunkMesh::get_vertex_ao(&padded_chunk, x, y + quad_y, z + quad_z, dx, dy, dz, ux, uy, uz);
        //             let ao3 = ChunkMesh::get_vertex_ao(&padded_chunk, x, y, z + quad_z, dx, dy, dz, -ux, -uy, -uz);

        //             let v1 = Vertex::new(x0, y0, z0, 0, ao0);
        //             let v2 = Vertex::new(x0, y1, z1, 0, ao2);
        //             let v3 = Vertex::new(x0, y1, z0, 0, ao1);
        //             let v4 = Vertex::new(x0, y0, z1, 0, ao3);

        //             // if is_left_face {
        //             //     vertices.extend_from_slice(&[v1, v2, v3, v1, v4, v2]);
        //             // } else {
        //             //     vertices.extend_from_slice(&[v1, v3, v2, v1, v2, v4]);
        //             // }

        //             let flip = ao0 + ao2 > ao1 + ao3;

        //             if !flip {
        //                 vertices.extend_from_slice(&[v1, v2, v3, v1, v4, v2]);
        //             } else {
        //                 vertices.extend_from_slice(&[v1, v3, v2, v1, v2, v4]);
        //             }

        //             // TODO: replace current architecture (vec of vertex) by the new one (vec of RenderFaceTexto)
        //             // Create a uniform buffer to store the chunk's coordinates of which we will draw the face
        //             // Change the shader to get the vertices by using x,y,z,w,h,direction
        //             // Long term: change the shader to use the texture property
        //             // WARNING: do not invert w & h.
        //             // Set width/height conventions for each axis that are both respected here and in the shader.
        //             // let data = RenderFaceTexto::new(
        //             //     x.to_u8).unwrap(),
        //             //     y.to_u8().unwrap(),
        //             //     z.to_u8().unwrap(),
        //             //     quad_z.to_u8().unwrap(),
        //             //     quad_y.to_u8().unwrap(),
        //             //     face.get_face(),
        //             //     0u16,
        //             // );

        //             // We can at least skip that part, knowing itering over this small part of the quad won't result in anything
        //             // Skipping quad_y will probably makes us lose vertex in the process, this is why we just skip z.
        //             z += quad_z;
        //         }
        //     }
        // }

        // // Y axis
        // for y in 0..=CHUNK_SIZE {
        //     // Mask
        //     for x in 0..CHUNK_SIZE {
        //         for z in 0..CHUNK_SIZE {
        //             previous = padded_chunk.get_block_from_xyz(x + 1, y, z + 1);
        //             current = padded_chunk.get_block_from_xyz(x + 1, y + 1, z + 1);

        //             match (previous.is_air(), current.is_air()) {
        //                 (true, true) | (false, false) => {
        //                     continue;
        //                 }
        //                 (true, false) => {
        //                     mask[x as usize][z as usize] =
        //                         FaceMask::from(false, current.id, Direction::Below);
        //                 }
        //                 (false, true) => {
        //                     mask[x as usize][z as usize] =
        //                         FaceMask::from(false, previous.id, Direction::Above);
        //                 }
        //             }
        //         }
        //     }

        //     // Mesh quads
        //     for x in 1..CHUNK_SIZE {
        //         let mut z = 1;
        //         while z < CHUNK_SIZE {
        //             let face = mask[x as usize][z as usize];
        //             if face.get_visited() {
        //                 z += 1;
        //                 continue;
        //             }

        //             mask[x as usize][z as usize].set_visited(true);

        //             let mut quad_x = 1i32;
        //             let mut quad_z = 1i32;

        //             // We grow the quad in the x-axis
        //             'outer: for ix in (x as usize + 1)..(CHUNK_SIZE as usize) as usize {
        //                 if mask[ix][z as usize].get_visited()
        //                     || mask[ix][z as usize].data != face.data
        //                 {
        //                     break 'outer;
        //                 }
        //                 quad_x += 1;
        //                 // Clear from the mask
        //                 mask[ix][z as usize].set_visited(true);
        //             }

        //             // We grow the quad in the z-axis
        //             'outer: for iz in (z + 1)..CHUNK_SIZE {
        //                 // We check if every face in the x is compatible with our expansion, and if not, we stop it
        //                 for ix in x..(x + quad_x) {
        //                     if mask[ix as usize][iz as usize].get_visited()
        //                         || mask[ix as usize][iz as usize].data != face.data
        //                     {
        //                         break 'outer;
        //                     }
        //                 }
        //                 quad_z += 1;
        //                 // Clear this space from the mask since we expand
        //                 for iy in (x as usize)..(x + quad_x) as usize {
        //                     mask[iy][iz as usize].set_visited(true);
        //                 }
        //             }

        //             // Add the quad to the mesh
        //             let is_above_face = face.get_face() == Direction::Above;

        //             let y0 = (y + offset_y) as f32;
        //             let x0 = (x + offset_x) as f32;
        //             let x1 = (x + quad_x + offset_x) as f32;
        //             let z0 = (z + offset_z) as f32;
        //             let z1 = (z + quad_z + offset_z) as f32;

        //             // directions pour AO
        //             let (dx, dy, dz) = (0, 0, 1); // côté “vertical” pour AO
        //             let (ux, uy, uz) = (1, 0, 0); // autre côté du quad

        //             let ao0 = ChunkMesh::get_vertex_ao(&padded_chunk, x, y, z, -dx, -dy, -dz, -ux, -uy, -uz);
        //             let ao1 = ChunkMesh::get_vertex_ao(&padded_chunk, x + quad_x, y, z, dx, dy, dz, ux, uy, uz);
        //             let ao2 = ChunkMesh::get_vertex_ao(&padded_chunk, x + quad_x, y, z + quad_z, dx, dy, dz, ux, uy, uz);
        //             let ao3 = ChunkMesh::get_vertex_ao(&padded_chunk, x, y, z + quad_z, -dx, -dy, -dz, -ux, -uy, -uz);

        //             let v1 = Vertex::new(x0, y0, z0, 0, 0);
        //             let v2 = Vertex::new(x1, y0, z1, 0, 0);
        //             let v3 = Vertex::new(x1, y0, z0, 0, 0);
        //             let v4 = Vertex::new(x0, y0, z1, 0, 0);

        //             // if is_above_face {
        //             //     vertices.extend_from_slice(&[v1, v2, v3, v1, v4, v2]);
        //             // } else {
        //             //     vertices.extend_from_slice(&[v1, v3, v2, v1, v2, v4]);
        //             // }

        //             let flip = ao0 + ao2 > ao1 + ao3;

        //             if !flip {
        //                 vertices.extend_from_slice(&[v1, v2, v3, v1, v4, v2]);
        //             } else {
        //                 vertices.extend_from_slice(&[v1, v3, v2, v1, v2, v4]);
        //             }

        //             // We can at least skip that part, knowing itering over this small part of the quad won't result in anything
        //             // Skipping quad_x will probably makes us lose vertex in the process, this is why we just skip z.
        //             z += quad_z;
        //         }
        //     }
        // }

        // // // Z axis
        // for z in 0..=CHUNK_SIZE {
        //     // Mask
        //     for x in 0..CHUNK_SIZE {
        //         for y in 0..CHUNK_SIZE {
        //             previous = padded_chunk.get_block_from_xyz(x + 1, y + 1, z);
        //             current = padded_chunk.get_block_from_xyz(x + 1, y + 1, z + 1);

        //             match (previous.is_air(), current.is_air()) {
        //                 (true, true) | (false, false) => {
        //                     continue;
        //                 }
        //                 (true, false) => {
        //                     mask[x as usize][y as usize] =
        //                         FaceMask::from(false, current.id, Direction::Back);
        //                 }
        //                 (false, true) => {
        //                     mask[x as usize][y as usize] =
        //                         FaceMask::from(false, previous.id, Direction::Front);
        //                 }
        //             }
        //         }
        //     }

        //     // Mesh quads
        //     for x in 1..CHUNK_SIZE {
        //         let mut y = 1;
        //         while y < CHUNK_SIZE {
        //             let face = mask[x as usize][y as usize];
        //             if face.get_visited() {
        //                 y += 1;
        //                 continue;
        //             }

        //             mask[x as usize][y as usize].set_visited(true);

        //             let mut quad_x = 1i32;
        //             let mut quad_y = 1i32;

        //             // We grow the quad in the x-axis
        //             'outer: for ix in (x as usize + 1)..(CHUNK_SIZE as usize) as usize {
        //                 if mask[ix][y as usize].get_visited()
        //                     || mask[ix][y as usize].data != face.data
        //                 {
        //                     break 'outer;
        //                 }
        //                 quad_x += 1;
        //                 // Clear from the mask
        //                 mask[ix][y as usize].set_visited(true);
        //             }

        //             // We grow the quad in the y-axis
        //             'outer: for iz in (y + 1)..CHUNK_SIZE {
        //                 // We check if every face in the x is compatible with our expansion, and if not, we stop it
        //                 for ix in x..(x + quad_x) {
        //                     if mask[ix as usize][iz as usize].get_visited()
        //                         || mask[ix as usize][iz as usize].data != face.data
        //                     {
        //                         break 'outer;
        //                     }
        //                 }
        //                 quad_y += 1;
        //                 // Clear this space from the mask since we expand
        //                 for iy in (x as usize)..(x + quad_x) as usize {
        //                     mask[iy][iz as usize].set_visited(true);
        //                 }
        //             }

        //             // Add the quad to the mesh
        //             let z0 = (z + offset_z) as f32;
        //             let x0 = (x + offset_x) as f32;
        //             let x1 = (x + quad_x + offset_x) as f32;
        //             let y0 = (y + offset_y) as f32;
        //             let y1 = (y + quad_y + offset_y) as f32;

        //             let (dx, dy, dz) = (1, 0, 0); // côté horizontal
        //             let (ux, uy, uz) = (0, 1, 0); // autre côté du quad

        //             let ao0 = ChunkMesh::get_vertex_ao(&padded_chunk, x, y, z, -dx, -dy, -dz, -ux, -uy, -uz);
        //             let ao1 = ChunkMesh::get_vertex_ao(&padded_chunk, x + quad_x, y, z, dx, dy, dz, ux, uy, uz);
        //             let ao2 = ChunkMesh::get_vertex_ao(&padded_chunk, x + quad_x, y + quad_y, z, dx, dy, dz, ux, uy, uz);
        //             let ao3 = ChunkMesh::get_vertex_ao(&padded_chunk, x, y + quad_y, z, -dx, -dy, -dz, -ux, -uy, -uz);

        //             let v1 = Vertex::new(x0, y0, z0, 0, ao0);
        //             let v2 = Vertex::new(x1, y0, z0, 0, ao2);
        //             let v3 = Vertex::new(x1, y1, z0, 0, ao1);
        //             let v4 = Vertex::new(x0, y1, z0, 0, ao3);

        //             let is_front = face.get_face() == Direction::Front;

        //             // if is_front {
        //             //     vertices.extend_from_slice(&[v1, v2, v3, v1, v3, v4]);
        //             // } else {
        //             //     vertices.extend_from_slice(&[v1, v3, v2, v1, v4, v3]);
        //             // }

        //             let flip = ao0 + ao2 > ao1 + ao3;

        //             if !flip {
        //                 vertices.extend_from_slice(&[v1, v2, v3, v1, v4, v2]);
        //             } else {
        //                 vertices.extend_from_slice(&[v1, v3, v2, v1, v2, v4]);
        //             }

        //             // We can at least skip that part, knowing itering over this small part of the quad won't result in anything
        //             // Skipping quad_x will probably makes us lose vertex in the process, this is why we just skip y.
        //             y += quad_y;
        //         }
        //     }
        // }

        self.dirty.store(false, Ordering::Relaxed);

        if let Some(mesh_id) = self.mesh_id {
            renderer.render_manager.update_mesh(&renderer.gpu_context.device, &renderer.gpu_context.queue, MeshData::new(vertices, None), mesh_id);
        }
        else {
            self.mesh_id = Some(renderer.render_manager.allocate_mesh(&renderer.gpu_context.device, &renderer.gpu_context.queue, MeshData::new(vertices, None)));
        }
    }
}

fn mul_i32_vec3(v: [i32; 3], scalar: i32) -> [i32; 3] {
    [v[0] * scalar, v[1] * scalar, v[2] * scalar]
}

fn add_i32_vec3(a: [i32; 3], b: [i32; 3]) -> [i32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}