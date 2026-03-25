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

    #[inline]
    fn is_solid(chunk: &PaddedChunk, x: i32, y: i32, z: i32) -> bool {
        // println!("{} {} {}", x, y, z);
        !chunk.get_block_from_xyz(x, y, z).is_air()
    }

    #[inline]
    fn vertex_ao(side1: bool, side2: bool, corner: bool) -> i32 {
        let value = if side1 && side2 {
            0
        } else {
            3 - (side1 as i32 + side2 as i32 + corner as i32)
        };

        print!("ao: {}, ", value);

        value
    }

    pub fn get_vertex_ao(
        chunk: &PaddedChunk,
        x: i32,
        y: i32,
        z: i32,
        dx: i32,
        dy: i32,
        dz: i32,
        ux: i32,
        uy: i32,
        uz: i32,
    ) -> i32 {

        let side1 = ChunkMesh::is_solid(chunk, x + dx, y + dy, z + dz);
        let side2 = ChunkMesh::is_solid(chunk, x + ux, y + uy, z + uz);
        let corner = ChunkMesh::is_solid(chunk, x + dx + ux, y + dy + uy, z + dz + uz);

        ChunkMesh::vertex_ao(side1, side2, corner)
    }

    fn build_pos(
        base: [i32; 3],
        e_d: [i32; 3],
        e_u: [i32; 3],
        e_v: [i32; 3],
        d: i32,
        u: i32,
        v: i32,
    ) -> [i32; 3] {
        [
            base[0] + d * e_d[0] + u * e_u[0] + v * e_v[0],
            base[1] + d * e_d[1] + u * e_u[1] + v * e_v[1],
            base[2] + d * e_d[2] + u * e_u[2] + v * e_v[2],
        ]
    }

    fn negate(v: [i32; 3]) -> [i32; 3] {
        [-v[0], -v[1], -v[2]]
    }

    const AO_TABLE: [u8; 8] = [
        3, 2, 2, 0,
        2, 1, 1, 0,
    ];

    fn get_vertex_ao_generic(
        chunk: &PaddedChunk,
        pos: [i32; 3],
        dir1: [i32; 3],
        dir2: [i32; 3],
    ) -> u8 {
        let s1 = chunk.get_block_from_chunk_xyz(
            pos[0] + dir1[0],
            pos[1] + dir1[1],
            pos[2] + dir1[2],
        ).is_solid() as usize;

        let s2 = chunk.get_block_from_chunk_xyz(
            pos[0] + dir2[0],
            pos[1] + dir2[1],
            pos[2] + dir2[2],
        ).is_solid() as usize;

        let c = chunk.get_block_from_chunk_xyz(
            pos[0] + dir1[0] + dir2[0],
            pos[1] + dir1[1] + dir2[1],
            pos[2] + dir1[2] + dir2[2],
        ).is_solid() as usize;

        ChunkMesh::AO_TABLE[s1 | (s2 << 1) | (c << 2)]
    }

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

        for d in 0..=LAST_PADDED_CHUNK_AXIS_INDEX {
            // === MASK BUILD ===
            for u in 0..=LAST_CHUNK_AXIS_INDEX {
                for v in 0..=LAST_CHUNK_AXIS_INDEX {
                    let pos = ChunkMesh::build_pos([0, 0, 0], e_d, e_u, e_v, d, u, v);
                    let pos_next = ChunkMesh::build_pos([0, 0, 0], e_d, e_u, e_v, d + 1, u, v);

                    let previous = padded_chunk.get_block_from_chunk_xyz(pos[0], pos[1], pos[2]);
                    let current = padded_chunk.get_block_from_chunk_xyz(pos_next[0], pos_next[1], pos_next[2]);

                    match (previous.is_air(), current.is_air()) {
                        (true, true) | (false, false) => {}
                        (true, false) => {
                            mask[u as usize][v as usize] =
                                FaceMask::from(false, current.id, Direction::Left);
                        }
                        (false, true) => {
                            mask[u as usize][v as usize] =
                                FaceMask::from(false, previous.id, Direction::Right);
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
                    let p0 = ChunkMesh::build_pos(base, e_d, e_u, e_v, d, u_i32, v_i32);
                    let p1 = ChunkMesh::build_pos(base, e_d, e_u, e_v, d, u_i32 + w_i32, v_i32);
                    let p2 = ChunkMesh::build_pos(base, e_d, e_u, e_v, d, u_i32 + w_i32, v_i32 + h_i32);
                    let p3 = ChunkMesh::build_pos(base, e_d, e_u, e_v, d, u_i32, v_i32 + h_i32);

                    // === AO directions ===
                    let du = e_u;
                    let dv = e_v;

                    let ao0 = ChunkMesh::get_vertex_ao_generic(padded_chunk, p0, ChunkMesh::negate(du), ChunkMesh::negate(dv));
                    let ao1 = ChunkMesh::get_vertex_ao_generic(padded_chunk, p1, du, ChunkMesh::negate(dv));
                    let ao2 = ChunkMesh::get_vertex_ao_generic(padded_chunk, p2, du, dv);
                    let ao3 = ChunkMesh::get_vertex_ao_generic(padded_chunk, p3, ChunkMesh::negate(du), dv);

                    let v1 = Vertex::new(p0[0] as f32, p0[1] as f32, p0[2] as f32, 0, ao0 as i32);
                    let v2 = Vertex::new(p2[0] as f32, p2[1] as f32, p2[2] as f32, 0, ao2 as i32);
                    let v3 = Vertex::new(p1[0] as f32, p1[1] as f32, p1[2] as f32, 0, ao1 as i32);
                    let v4 = Vertex::new(p3[0] as f32, p3[1] as f32, p3[2] as f32, 0, ao3 as i32);

                    let flip = ao0 + ao2 > ao1 + ao3;

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
