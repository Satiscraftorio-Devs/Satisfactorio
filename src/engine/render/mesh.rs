use std::{collections::HashMap, sync::Arc};

use cgmath::num_traits::ToPrimitive;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{player::Player, world::{Block, CHUNK_SIZE, CHUNK_SIZE_SQR, Chunk, LAST_CHUNK_BLOCK_INDEX, World}};
use crate::engine::render::geometry::Vertex;

const X: f32 = 1.0;
const Y: f32 = 1.0;
const Z: f32 = 1.0;

// Block's vertices positions (counter clockwise)
// Face arrière d'un cube, en partant du coin inférieur gauche
const V_000: [f32; 3] = [0.0, 0.0, 0.0];
const V_100: [f32; 3] = [ X , 0.0, 0.0];
const V_110: [f32; 3] = [ X ,  Y , 0.0];
const V_010: [f32; 3] = [0.0,  Y , 0.0];
// Face devant la première, en partant du coin inférieur gauche
const V_001: [f32; 3] = [0.0, 0.0,  Z ];
const V_101: [f32; 3] = [ X , 0.0,  Z ];
const V_111: [f32; 3] = [ X ,  Y ,  Z ];
const V_011: [f32; 3] = [0.0,  Y ,  Z ];

#[repr(u8)]
#[derive(PartialEq)]
#[derive(Clone, Copy)]
enum Face {
    Above,
    Below,
    Left,
    Right,
    Front,
    Back,
}

pub struct ChunkMesh {
    pub vertices: Vec<Vertex>,
    dirty: bool,
}

impl ChunkMesh {
    fn new() -> ChunkMesh {
        return ChunkMesh {
            vertices: vec![],
            dirty: true
        };
    }

    fn add_behind_face(&mut self, x: f32, y: f32, z: f32) {
        self.vertices.extend_from_slice(&[
            Vertex::new(V_100[0] + x, V_100[1] + y, V_100[2] + z),
            Vertex::new(V_000[0] + x, V_000[1] + y, V_000[2] + z),
            Vertex::new(V_110[0] + x, V_110[1] + y, V_110[2] + z),
            Vertex::new(V_110[0] + x, V_110[1] + y, V_110[2] + z),
            Vertex::new(V_000[0] + x, V_000[1] + y, V_000[2] + z),
            Vertex::new(V_010[0] + x, V_010[1] + y, V_010[2] + z)
        ]);
    }

    fn add_front_face(&mut self, x: f32, y: f32, z: f32) {
        self.vertices.extend_from_slice(&[
            Vertex::new(V_001[0] + x, V_001[1] + y, V_001[2] + z),
            Vertex::new(V_101[0] + x, V_101[1] + y, V_101[2] + z),
            Vertex::new(V_011[0] + x, V_011[1] + y, V_011[2] + z),
            Vertex::new(V_011[0] + x, V_011[1] + y, V_011[2] + z),
            Vertex::new(V_101[0] + x, V_101[1] + y, V_101[2] + z),
            Vertex::new(V_111[0] + x, V_111[1] + y, V_111[2] + z)
        ]);
    }

    fn add_left_face(&mut self, x: f32, y: f32, z: f32) {
        self.vertices.extend_from_slice(&[
            Vertex::new(V_000[0] + x, V_000[1] + y, V_000[2] + z),
            Vertex::new(V_001[0] + x, V_001[1] + y, V_001[2] + z),
            Vertex::new(V_010[0] + x, V_010[1] + y, V_010[2] + z),
            Vertex::new(V_010[0] + x, V_010[1] + y, V_010[2] + z),
            Vertex::new(V_001[0] + x, V_001[1] + y, V_001[2] + z),
            Vertex::new(V_011[0] + x, V_011[1] + y, V_011[2] + z)
        ]);
    }

    fn add_right_face(&mut self, x: f32, y: f32, z: f32) {
        self.vertices.extend_from_slice(&[
            Vertex::new(V_100[0] + x, V_100[1] + y, V_100[2] + z),
            Vertex::new(V_110[0] + x, V_110[1] + y, V_110[2] + z),
            Vertex::new(V_101[0] + x, V_101[1] + y, V_101[2] + z),
            Vertex::new(V_101[0] + x, V_101[1] + y, V_101[2] + z),
            Vertex::new(V_110[0] + x, V_110[1] + y, V_110[2] + z),
            Vertex::new(V_111[0] + x, V_111[1] + y, V_111[2] + z)
        ]);
    }

    fn add_above_face(&mut self, x: f32, y: f32, z: f32) {
        self.vertices.extend_from_slice(&[
            Vertex::new(V_010[0] + x, V_010[1] + y, V_010[2] + z),
            Vertex::new(V_011[0] + x, V_011[1] + y, V_011[2] + z),
            Vertex::new(V_110[0] + x, V_110[1] + y, V_110[2] + z),
            Vertex::new(V_110[0] + x, V_110[1] + y, V_110[2] + z),
            Vertex::new(V_011[0] + x, V_011[1] + y, V_011[2] + z),
            Vertex::new(V_111[0] + x, V_111[1] + y, V_111[2] + z)
        ]);
    }

    fn add_below_face(&mut self, x: f32, y: f32, z: f32) {
        self.vertices.extend_from_slice(&[
            Vertex::new(V_000[0] + x, V_000[1] + y, V_000[2] + z),
            Vertex::new(V_100[0] + x, V_100[1] + y, V_100[2] + z),
            Vertex::new(V_001[0] + x, V_001[1] + y, V_001[2] + z),
            Vertex::new(V_001[0] + x, V_001[1] + y, V_001[2] + z),
            Vertex::new(V_100[0] + x, V_100[1] + y, V_100[2] + z),
            Vertex::new(V_101[0] + x, V_101[1] + y, V_101[2] + z)
        ]);
    }

    pub fn make(chunk: &Chunk, world: &World, cx: i32, cy: i32, cz: i32) -> ChunkMesh {
        // C'est là que ça devient vraiment rigolo
        let mut mesh = ChunkMesh::new();

        let offset_x = cx * CHUNK_SIZE;
        let offset_y = cy * CHUNK_SIZE;
        let offset_z = cz * CHUNK_SIZE;

        let mut block: Block;
        for lz in 0..CHUNK_SIZE {
            let iz = lz * CHUNK_SIZE_SQR;
            for ly in 0..CHUNK_SIZE {
                let iy = ly * CHUNK_SIZE;
                for lx in 0..CHUNK_SIZE {
                    let li = (lx + iy + iz) as usize;
                    block = chunk.get_block_from_i(li);
                    if block.is_air() {
                        continue;
                    }
                    
                    let gx = lx as i32 + offset_x;
                    let gy = ly as i32 + offset_y;
                    let gz = lz as i32 + offset_z;
                    
                    // println!("{gx} {gy} {gz} ");

                    // Check des voisins intra-chunk
                    let left = if lx == 0 { 
                        world.get_block_from_xyz(gx-1, gy, gz)
                    }
                    else {
                        chunk.get_block_from_xyz(lx-1, ly, lz)
                    };
                    let right = if lx == LAST_CHUNK_BLOCK_INDEX {
                        world.get_block_from_xyz(gx+1, gy, gz)
                    }
                    else {
                        chunk.get_block_from_xyz(lx+1, ly, lz)
                    };
                    let below = if ly == 0 {
                        world.get_block_from_xyz(gx, gy-1, gz)
                    }
                    else {
                        chunk.get_block_from_xyz(lx, ly-1, lz)
                    };
                    let above = if ly == LAST_CHUNK_BLOCK_INDEX {
                        world.get_block_from_xyz(gx, gy+1, gz)
                    }
                    else {
                        chunk.get_block_from_xyz(lx, ly+1, lz)
                    };
                    let behind = if lz == 0 {
                        world.get_block_from_xyz(gx, gy, gz-1)
                    }
                    else {
                        chunk.get_block_from_xyz(lx, ly, lz-1)
                    };
                    let front = if lz == LAST_CHUNK_BLOCK_INDEX {
                        world.get_block_from_xyz(gx, gy, gz+1)
                    }
                    else {
                        chunk.get_block_from_xyz(lx, ly, lz+1)
                    };
                    
                    if front.is_air() {
                        // print!("f ");
                        mesh.add_front_face(gx.to_f32().unwrap(), gy.to_f32().unwrap(), gz.to_f32().unwrap());
                    }

                    if behind.is_air() {
                        // print!("bh ");
                        mesh.add_behind_face(gx.to_f32().unwrap(), gy.to_f32().unwrap(), gz.to_f32().unwrap());
                    }

                    if left.is_air() {
                        // print!("l ");
                        mesh.add_left_face(gx.to_f32().unwrap(), gy.to_f32().unwrap(), gz.to_f32().unwrap());
                    }

                    if right.is_air() {
                        // print!("r ");
                        mesh.add_right_face(gx.to_f32().unwrap(), gy.to_f32().unwrap(), gz.to_f32().unwrap());
                    }

                    if above.is_air() {
                        // print!("a ");
                        mesh.add_above_face(gx.to_f32().unwrap(), gy.to_f32().unwrap(), gz.to_f32().unwrap());
                    }

                    if below.is_air() {
                        // print!("bl ");
                        mesh.add_below_face(gx.to_f32().unwrap(), gy.to_f32().unwrap(), gz.to_f32().unwrap());
                    }
                }
            }
        }

        mesh.dirty = false;

        return mesh;
    }

    pub fn make_greedy(chunk: &Chunk, world: &World, cx: i32, cy: i32, cz: i32) -> ChunkMesh {
        let mut mesh = ChunkMesh::new();

        let offset_x = cx * CHUNK_SIZE;
        let offset_y = cy * CHUNK_SIZE;
        let offset_z = cz * CHUNK_SIZE;

        // We allocate once to avoid memory reallocation/destruction.
        let mut mask: [[Option<(i32, Face)>; CHUNK_SIZE as usize]; CHUNK_SIZE as usize] = [[None; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
        
        let mut current: Block;
        let mut next: Block;
        let mut is_current_air: bool;
        let mut is_next_air: bool;

        // Todo:
        // - replace world.get_block by chunk.get_block whenever possible
        // - check if the block is in the chunk before adding it to the mask
        
        // X axis
        for slice in -1..CHUNK_SIZE {
            // Mask
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    current = world.get_block_from_xyz(slice + offset_x, y + offset_y, z + offset_z);
                    next = world.get_block_from_xyz(slice + offset_x + 1, y + offset_y, z + offset_z);

                    is_current_air = current.is_air();
                    is_next_air = next.is_air();

                    if is_current_air == is_next_air {
                        mask[y as usize][z as usize] = None;
                        continue;
                    }
                    
                    if is_current_air && !is_next_air {
                        mask[y as usize][z as usize] = Some((next.id, Face::Left));
                    }
                    else {
                        mask[y as usize][z as usize] = Some((current.id, Face::Right));
                    }
                }
            }

            // Mesh quads
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let Some(face) = mask[y as usize][z as usize] else {
                        continue;
                    };

                    let mut quad_y = 1;
                    let mut quad_z = 1;

                    // We grow the quad in the y-axis
                    'outer: for iy in (y+1)..CHUNK_SIZE {
                        let Some(y_neighbor) = mask[iy as usize][z as usize] else {
                            break 'outer;
                        };
                        if y_neighbor.0 != face.0 || y_neighbor.1 != face.1 {
                            break 'outer;
                        }
                        quad_y += 1;
                    }

                    // We grow the quad in the z-axis
                    'outer: for iz in (z+1)..CHUNK_SIZE {
                        // We check if every face in the y is compatible with our expansion, and if not, we stop it
                        for iy in y..(y + quad_y) {
                            let Some(z_neighbor) = mask[iy as usize][iz as usize] else {
                                break 'outer;
                            };
                            if z_neighbor.0 != face.0 || z_neighbor.1 != face.1 {
                                break 'outer;
                            }
                        }
                        quad_z += 1;
                    }

                    // Add the quad to the mesh
                    if face.1 == Face::Right {
                        let x = slice as f32 + offset_x as f32;
                        let y0 = y as f32 + offset_y as f32;
                        let y1 = (y + quad_y) as f32 + offset_y as f32;
                        let z0 = z as f32 + offset_z as f32;
                        let z1 = (z + quad_z) as f32 + offset_z as f32;

                        mesh.vertices.extend_from_slice(&[
                            Vertex::new(x, y0, z0),
                            Vertex::new(x, y1, z0),
                            Vertex::new(x, y1, z1),

                            Vertex::new(x, y0, z0),
                            Vertex::new(x, y1, z1),
                            Vertex::new(x, y0, z1),
                        ]);
                    }
                    else if face.1 == Face::Left {
                        let x = slice as f32 + offset_x as f32 + 1.0;
                        let y0 = y as f32 + offset_y as f32;
                        let y1 = (y + quad_y) as f32 + offset_y as f32;
                        let z0 = z as f32 + offset_z as f32;
                        let z1 = (z + quad_z) as f32 + offset_z as f32;

                        mesh.vertices.extend_from_slice(&[
                            Vertex::new(x, y0, z0),
                            Vertex::new(x, y1, z1),
                            Vertex::new(x, y1, z0),

                            Vertex::new(x, y0, z0),
                            Vertex::new(x, y0, z1),
                            Vertex::new(x, y1, z1),
                        ]);
                    }

                    // Clear the quad from the mask to avoid vertex dupplication
                    for iy in y..(y + quad_y) {
                        for iz in z..(z + quad_z) {
                            mask[iy as usize][iz as usize] = None;
                        }
                    }
                }
            }
        }

        // Y axis

        // Z axis

        return mesh;
    }
}

pub struct WorldMesh {
    pub meshes: HashMap<(i32,i32,i32), Arc<ChunkMesh>>
}

impl WorldMesh {
    pub fn new() -> WorldMesh {
        return WorldMesh {
            meshes: HashMap::new(),
        };
    }

    /// Builds simultaneously every single chunk within the player's both horizontal and vertical render distance only if it needs it (if dirty == true).
    pub fn update(world: &World, player: &Player, old: &WorldMesh) -> WorldMesh {
        let meshes: HashMap<_, _> = world.get_player_rendered_chunks(player)
            .into_par_iter()
            .map(|(chunk, cx, cy, cz)| {
                let key = (cx, cy, cz);

                if let Some(existing) = old.meshes.get(&key) {
                    if !existing.dirty {
                        return (key, Arc::clone(existing));
                    }
                }

                let mesh = ChunkMesh::make_greedy(chunk, world, cx, cy, cz);
                // let mesh = ChunkMesh::make(chunk, world, cx, cy, cz);
                return (key, Arc::new(mesh));
            })
            .collect();

        return WorldMesh {
            meshes
        };
    }
}