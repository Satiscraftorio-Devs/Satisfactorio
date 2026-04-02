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

impl Corner {
    pub fn get_opposite_horizontal(&self) -> Self {
        match *self {
            Self::BottomLeft => Self::BottomRight,
            Self::BottomRight => Self::BottomLeft,
            Self::TopLeft => Self::TopRight,
            Self::TopRight => Self::TopLeft,
        }
    }
}

pub struct ChunkMesh {
    pub mesh_id: Option<MeshId>,
    dirty: AtomicBool,
}

impl ChunkMesh {
    pub fn new() -> ChunkMesh {
        return ChunkMesh {
            mesh_id: None,
            dirty: AtomicBool::new(true),
        };
    }

    pub fn set_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    pub fn is_dirty(&self) -> bool {
        return self.dirty.load(Ordering::Relaxed);
    }

    pub fn get_v_ao(chunk: &PaddedChunk, pos: Vector3<i32>, neighbors: [(i32, i32, i32); 3]) -> u8 {
        let corner_solid = chunk
            .get_block_from_chunk_xyz(pos[0] + neighbors[0].0, pos[1] + neighbors[0].1, pos[2] + neighbors[0].2)
            .is_solid() as u8;
        let side1_solid = chunk
            .get_block_from_chunk_xyz(pos[0] + neighbors[1].0, pos[1] + neighbors[1].1, pos[2] + neighbors[1].2)
            .is_solid() as u8;
        let side2_solid = chunk
            .get_block_from_chunk_xyz(pos[0] + neighbors[2].0, pos[1] + neighbors[2].1, pos[2] + neighbors[2].2)
            .is_solid() as u8;

        if side1_solid == 1 && side2_solid == 1 {
            return 0;
        } else {
            return 3 - (side1_solid + side2_solid + corner_solid);
        }
    }

    fn get_ao_offsets(face: Direction, corner: Corner) -> [(i32, i32, i32); 3] {
        let (u, v, normal) = match face {
            Direction::Left => ((0, 1, 0), (0, 0, 1), (-1, 0, 0)),
            Direction::Below => ((1, 0, 0), (0, 0, 1), (0, -1, 0)),
            Direction::Back => ((1, 0, 0), (0, 1, 0), (0, 0, -1)),
            Direction::Right => ((0, 1, 0), (0, 0, 1), (1, 0, 0)),
            Direction::Above => ((1, 0, 0), (0, 0, 1), (0, 1, 0)),
            Direction::Front => ((1, 0, 0), (0, 1, 0), (0, 0, 1)),
        };

        let (u, v) = match face {
            Direction::Above | Direction::Below => (u, v),
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

    pub fn make_greedy_axis(padded_chunk: &PaddedChunk, vertices: &mut Vec<Vertex>, cx: i32, cy: i32, cz: i32, axis: i32) {
        // if axis != 1 {
        //     return;
        // }

        let base = Vector3::new(cx * CHUNK_SIZE, cy * CHUNK_SIZE, cz * CHUNK_SIZE);

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

        let faces: [Direction; 2] = match axis {
            0 => [Direction::Right, Direction::Left],
            1 => [Direction::Above, Direction::Below],
            2 => [Direction::Front, Direction::Back],
            _ => unreachable!(),
        };

        for d in 0..=CHUNK_SIZE {
            for u in 0..=LAST_CHUNK_AXIS_INDEX {
                for v in 0..=LAST_CHUNK_AXIS_INDEX {
                    let previous_pos = e_d * (d - 1) + e_u * u + e_v * v;
                    let current_pos = e_d * d + e_u * u + e_v * v;

                    let previous = padded_chunk.get_block_from_chunk_xyz(previous_pos[0], previous_pos[1], previous_pos[2]);
                    let current = padded_chunk.get_block_from_chunk_xyz(current_pos[0], current_pos[1], current_pos[2]);

                    match (previous.is_solid(), current.is_solid()) {
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

                            mask[u as usize][v as usize] = FaceMask::from(
                                false,
                                current.id,
                                match axis {
                                    0 => Direction::Left,
                                    1 => Direction::Below,
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

                            mask[u as usize][v as usize] = FaceMask::from(
                                false,
                                previous.id,
                                match axis {
                                    0 => Direction::Right,
                                    1 => Direction::Above,
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

                    for iu in u..=LAST_CHUNK_AXIS_INDEX_USIZE {
                        if mask[iu][v].get_visited() || !mask[iu][v].can_merge_with(&face) {
                            break;
                        }
                        width += 1;
                        mask[iu][v].set_visited(true);
                    }

                    'expand: for iv in v..=LAST_CHUNK_AXIS_INDEX_USIZE {
                        for iu in u..(u + width) {
                            if mask[iu][iv].get_visited() || !mask[iu][iv].can_merge_with(&face) {
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

                    let block_pos = base + e_v * v_i32 + e_u * u_i32 + e_d * d;

                    let e_u_w = e_u * w_i32;
                    let e_v_h = e_v * h_i32;
                    let e_uv_wh = e_u_w + e_v_h;

                    let local_position_v0 = block_pos;
                    let local_position_v1 = block_pos + e_v_h;
                    let local_position_v2 = block_pos + e_u_w;
                    let local_position_v3 = block_pos + e_uv_wh;

                    let vertex_0_ao = face.get_ao() >> 6;
                    let vertex_1_ao = (face.get_ao() >> 4) & 0b11;
                    let vertex_2_ao = (face.get_ao() >> 2) & 0b11;
                    let vertex_3_ao = face.get_ao() & 0b11;
                    // let vertex_0_ao = mask[u][v].get_ao_corner(2);
                    // let vertex_1_ao = mask[u][v].get_ao_corner(3);
                    // let vertex_2_ao = mask[u + width - 1][v].get_ao_corner(0);
                    // let vertex_3_ao = mask[u + width - 1][v].get_ao_corner(1);

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

                    let reverse_faces = face.get_face().is_negative();

                    if reverse_faces {
                        vertices.extend_from_slice(&[v0, v1, v2, v2, v1, v3]);
                    } else {
                        vertices.extend_from_slice(&[v1, v0, v3, v3, v0, v2]);
                    }

                    v += height;
                }
            }
        }
    }

    pub fn make_greedy(&mut self, chunk: &Chunk, world: &World, renderer: &mut Renderer, cx: i32, cy: i32, cz: i32) {
        let mut vertices: Vec<Vertex> = vec![];
        let padded_chunk = PaddedChunk::new(chunk, world);

        ChunkMesh::make_greedy_axis(&padded_chunk, &mut vertices, cx, cy, cz, 0);
        ChunkMesh::make_greedy_axis(&padded_chunk, &mut vertices, cx, cy, cz, 1);
        ChunkMesh::make_greedy_axis(&padded_chunk, &mut vertices, cx, cy, cz, 2);

        self.dirty.store(false, Ordering::Relaxed);

        if let Some(mesh_id) = self.mesh_id {
            renderer.render_manager.update_mesh(
                &renderer.gpu_context.device,
                &renderer.gpu_context.queue,
                MeshData::new(vertices, None),
                mesh_id,
            );
        } else {
            self.mesh_id = Some(renderer.render_manager.allocate_mesh(
                &renderer.gpu_context.device,
                &renderer.gpu_context.queue,
                MeshData::new(vertices, None),
            ));
        }
    }
}
