use std::collections::HashMap;

use wgpu::{Device, Queue};

use crate::engine::render::mesh::mesh::{Mesh, MeshData, MeshId};

pub struct RenderManager {
    meshes: HashMap<MeshId, Mesh>,
    mesh_pool: Vec<Mesh>,
    id_pool: Vec<MeshId>,
    next_id: MeshId,
    ids_to_render: Vec<MeshId>,
}

impl RenderManager {
    pub fn new() -> Self {
        Self {
            meshes: HashMap::new(),
            mesh_pool: vec![],
            id_pool: vec![],
            next_id: 0,
            ids_to_render: vec![],
        }
    }

    fn get_next_id(&mut self) -> MeshId {
        if let Some(id) = self.id_pool.pop() {
            return id;
        }

        let id = self.next_id;
        self.next_id += 1;

        id
    }

    pub fn allocate_mesh(&mut self, device: &Device, queue: &Queue, data: MeshData) -> MeshId {
        let id = self.get_next_id();

        let mesh = {
            if let Some(mut mesh) = self.mesh_pool.pop() {
                mesh.update(device, queue, data);
                mesh
            } else {
                Mesh::new(device, queue, data)
            }
        };
        self.meshes.insert(id, mesh);

        // println!("Affected mesh with id: {} mesh count: {}", id, self.meshes.len());

        id
    }

    pub fn update_mesh(&mut self, device: &Device, queue: &Queue, data: MeshData, id: MeshId) -> bool {
        if let Some(mesh) = self.meshes.get_mut(&id) {
            mesh.update(device, queue, data);
            return true;
        }
        return false;
    }

    pub fn release_mesh(&mut self, id: MeshId) {
        if let Some(mesh) = self.meshes.remove(&id) {
            self.mesh_pool.push(mesh);
            self.id_pool.push(id);
        }
    }

    pub fn mark_mesh_for_rendering(&mut self, id: MeshId) {
        if self.meshes.contains_key(&id) {
            self.ids_to_render.push(id);
        }
    }

    pub fn get_meshes_to_render(&self) -> Vec<&Mesh> {
        self.ids_to_render.iter().filter_map(|id| self.meshes.get(id)).collect()
    }

    pub fn clear_render_queue(&mut self) {
        self.ids_to_render.clear();
    }
}
