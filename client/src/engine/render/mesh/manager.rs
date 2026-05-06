use std::{time::Instant, usize};

use shared::log_client;
use wgpu::{Buffer, BufferUsages, CommandEncoder, Device, Queue};

use crate::engine::render::utils::smart_buffer::SmartBuffer;

const BYTES_PER_FRAME_CAP: usize = 1024 * 1024 * 8;
const MAX_MILLIS_PER_FRAME_CAP: u128 = 8;
const MAX_WRITE_OPERATIONS_PER_FRAME: usize = 5;
const ARENA_MIN_SIZE: usize = 1024 * 1024 * 16;

pub type MeshId = u32;

pub struct DataEntry<'a> {
    pub id: MeshId,
    pub data: &'a [u8],
}

impl<'a> DataEntry<'a> {
    pub fn new(id: MeshId, data: &'a [u8]) -> Self {
        Self { id, data }
    }
}

pub struct MeshEntry {
    pub id: MeshId,
    pub position: usize,
    pub length: usize,
}

#[derive(Clone)]
struct Gap {
    pub position: usize,
    pub length: usize,
}

struct WriteOperation {
    mesh_id: MeshId,
    offset: usize,
    len: usize,
    arena_offset: usize,
}

pub struct MeshManager {
    buffer: SmartBuffer,
    pub data: Vec<MeshEntry>,
    gaps: Vec<Gap>,
    pending_destruction: Vec<SmartBuffer>,
    next_id: MeshId,
    free_ids: Vec<MeshId>,
    write_operations: Vec<WriteOperation>,
    schedule_merge: bool,
    arena: Vec<u8>,
}

impl Gap {
    pub fn new(position: usize, length: usize) -> Self {
        Self { position, length }
    }
}

impl MeshManager {
    pub fn new(device: &Device) -> Self {
        let mut arena = Vec::new();
        arena.reserve(ARENA_MIN_SIZE);
        Self {
            buffer: SmartBuffer::from_capacity(
                0,
                device,
                None,
                BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::VERTEX,
            ),
            data: vec![],
            gaps: vec![],
            pending_destruction: vec![],
            next_id: 0,
            free_ids: Vec::with_capacity(64),
            write_operations: vec![],
            schedule_merge: false,
            arena: arena,
        }
    }

    pub fn get_buffer(&self) -> &Buffer {
        self.buffer.buffer()
    }

    fn reallocate_defragment(&mut self, device: &Device, queue: &Queue, encoder: &mut CommandEncoder, needed: usize) {
        if !self.write_operations.is_empty() {
            log_client!("REALLOCATE DEFRAGMENT BEFORE FLUSH");
            self.flush(queue);
        }
        println!("Reallocate + defragment - needed: {}", needed);

        // Reallocate
        let new_buffer = SmartBuffer::from_capacity(
            needed as u32,
            device,
            None,
            BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::VERTEX,
        );

        let mut current_position = 0;

        // Copy each entry to the new buffer without any gaps
        for entry in &mut self.data {
            encoder.copy_buffer_to_buffer(
                self.buffer.buffer(),
                entry.position as u64,
                new_buffer.buffer(),
                current_position as u64,
                entry.length as u64,
            );

            // Update entry
            entry.position = current_position;

            current_position += entry.length;
        }

        // Update gaps
        self.gaps.clear();
        println!(
            "new realloc gap: {} {}",
            current_position,
            new_buffer.capacity() as usize - current_position
        );
        self.gaps
            .push(Gap::new(current_position, new_buffer.capacity() as usize - current_position));

        // Update buffer
        self.pending_destruction.push(std::mem::replace(&mut self.buffer, new_buffer));

        self.print_buffer_infos();
    }

    fn find_place(&self, needed: usize) -> Option<usize> {
        println!("Finding place for {} bytes.", needed);
        self.gaps.iter().position(|x| x.length >= needed)
    }

    fn get_data_next_gap(&self, entry: &MeshEntry) -> Option<usize> {
        println!(
            "Getting data next gap for Mesh(id: {}, pos: {}, len: {}).",
            entry.id, entry.position, entry.length
        );
        let position = entry.position + entry.length;
        self.gaps.iter().position(|gap| gap.position == position)
    }

    fn write_at(&mut self, position: usize, data: &[u8], mesh_id: MeshId) {
        println!("Writing at {} data of len {} and of Mesh(id: {})", position, data.len(), mesh_id);
        let arena_offset = self.arena.len();
        self.arena.extend_from_slice(data);
        self.write_operations.push(WriteOperation {
            mesh_id,
            offset: position,
            len: data.len(),
            arena_offset: arena_offset,
        });
        self.schedule_merge = true;
    }

    fn merge_commands(&mut self) {
        self.write_operations.sort_by_key(|op| op.offset);

        let mut merged: Vec<WriteOperation> = Vec::with_capacity(self.write_operations.len());

        for op in self.write_operations.drain(..) {
            if let Some(last) = merged.last_mut() {
                if last.offset + last.len == op.offset {
                    last.len += op.len;
                    continue;
                }
            }
            merged.push(op);
        }

        self.write_operations = merged;
    }

    pub fn flush(&mut self, queue: &Queue) {
        if self.write_operations.is_empty() {
            return;
        }
        println!("Flushing!");

        if self.schedule_merge {
            // self.merge_commands();
            self.schedule_merge = false;
        }

        let mut used_bytes = 0;
        let mut used_ops = 0;
        let mut time_took_millis = 0;
        let mut i = 0;

        while i < self.write_operations.len() {
            let op = &self.write_operations[i];

            // if time_took_millis >= MAX_MILLIS_PER_FRAME_CAP
            //     || used_bytes >= BYTES_PER_FRAME_CAP
            //     || used_ops >= MAX_WRITE_OPERATIONS_PER_FRAME
            // {
            //     break;
            // }

            used_bytes += op.len;
            used_ops += 1;

            let now = Instant::now();

            queue.write_buffer(
                self.buffer.buffer(),
                op.offset as u64,
                &self.arena[op.arena_offset..op.arena_offset + op.len],
            );

            time_took_millis += now.elapsed().as_millis();

            i += 1;
        }

        self.write_operations.drain(0..i);

        if self.write_operations.is_empty() {
            self.arena.clear();
        }
    }

    fn insert_gap_after(&mut self, gap: Gap, position: usize) {
        println!("Inserting Gap(pos: {}, len: {}) after pos: {}.", gap.position, gap.length, position);
        let gap_index = self.gaps.iter().position(|x| x.position > position).unwrap_or(self.gaps.len());

        self.gaps.insert(gap_index, gap);
    }

    pub fn process_pending_destructions(&mut self) {
        if self.pending_destruction.is_empty() {
            return;
        }
        println!("Process pending destructions.");
        for mut buf in self.pending_destruction.drain(..) {
            buf.destroy();
        }
    }

    fn consume_gap_for(&mut self, gap_index: usize, entry: &DataEntry) {
        println!(
            "Consume Gap(index: {}) for DataEntry(id: {}, len: {}).",
            gap_index,
            entry.id,
            entry.data.len()
        );
        let position = self.gaps[gap_index].position;
        let data_length = entry.data.len();

        self.write_at(position, entry.data, entry.id);

        let gap = &mut self.gaps[gap_index];
        gap.position += data_length;
        gap.length -= data_length;

        if gap.length == 0 {
            self.gaps.remove(gap_index);
        }
    }

    pub fn update_data(&mut self, device: &Device, queue: &Queue, encoder: &mut CommandEncoder, entry: DataEntry) {
        println!("Updating data for DataEntry(id: {}, len: {}).", entry.id, entry.data.len());
        let Some(index) = self.data.iter().position(|x| x.id == entry.id) else {
            return;
        };

        let (position, old_len) = {
            let entry = &self.data[index];
            (entry.position, entry.length)
        };
        let new_len = entry.data.len();

        // Cas 1: les nouvelles données ont une taille inférieure ou égale aux précédentes
        if new_len <= old_len {
            self.write_at(position, entry.data, entry.id);

            if new_len < old_len {
                if let Some(gap_index) = self.get_data_next_gap(&self.data[index]) {
                    let gap = &mut self.gaps[gap_index];

                    let delta_length = old_len - new_len;
                    gap.position -= delta_length;
                    gap.length += delta_length;
                } else {
                    let gap = Gap::new(position + new_len, old_len - new_len);
                    self.insert_gap_after(gap, position);
                }
            }

            self.data[index].length = new_len;

            return;
        }

        // Cas 2: on regarde si la taille des anciennes données + le trou qui les suit est suffisant pour accueillir les nouvelles données
        let gap_index = self.gaps.iter().position(|x| x.position == position + old_len);

        // S'il y a un trou après
        if let Some(gap_index) = gap_index {
            let gap_length = self.gaps[gap_index].length;

            // Si taille(anciennes données + trou) suffisent
            if new_len <= old_len + gap_length {
                self.write_at(position, entry.data, entry.id);

                let gap = &mut self.gaps[gap_index];
                gap.position = position + new_len;
                gap.length = old_len + gap.length - new_len;

                if gap.length == 0 {
                    self.gaps.remove(gap_index);
                }

                self.data[index].length = new_len;

                return;
            }
            // Si ça suffit pas, on élargit le trou à (ancienne données + trou actuel) et donc on marque l'emplacement comme libre
            else {
                let gap = &mut self.gaps[gap_index];
                gap.position = position;
                gap.length += old_len;

                self.data.remove(index);
            }
        }
        // S'il n'y a pas de trou après les données actuelle, on a pas la place pour stocker les nouvelles. On marque alors l'emplacement actuel comme libre
        else {
            let gap = Gap::new(position, old_len);
            self.insert_gap_after(gap, position);

            self.data.remove(index);
        }

        // Cas 3: on regarde s'il existe un trou suffisant pour accueillir les nouvelles données...
        if let Some(gap_index) = self.find_place(new_len) {
            self.consume_gap_for(gap_index, &entry);
            return;
        }

        // Cas 4: c'est la merde, donc on réalloue et défragmente pour avoir assez d'espace pour les nouvelles données (DERNIER RECOURS)
        let needed = self.data.iter().fold(0, |acc, x| acc + x.length) + new_len;
        self.reallocate_defragment(device, queue, encoder, needed);

        let gap_index = self
            .find_place(new_len)
            .expect("Failed to update data. No free space found even after reallocation.");
        self.consume_gap_for(gap_index, &entry);

        self.print_buffer_infos();
    }

    pub fn get_new_id(&mut self) -> MeshId {
        self.free_ids.pop().unwrap_or_else(|| {
            let id = self.next_id;
            self.next_id += 1;
            id
        })
    }

    pub fn add_data(&mut self, device: &Device, queue: &Queue, encoder: &mut CommandEncoder, data: &[u8]) -> Option<MeshId> {
        println!("Adding data for data of len: {}.", data.len());
        let mut index = self.find_place(data.len());

        if index.is_none() {
            let needed = self.data.iter().fold(0, |acc, x| acc + x.length) + data.len();
            self.reallocate_defragment(device, queue, encoder, needed);

            index = self.find_place(data.len());
        }

        let index = index.expect(
            format!(
                "No space found, even after re-allocation.\nData needed {} bytes but buffer is {} bytes long and has {} bytes of capacity.",
                data.len(),
                self.buffer.length(),
                self.buffer.capacity(),
            )
            .as_str(),
        );

        let (position, gap_length) = {
            let gap = &mut self.gaps[index];

            let pos = gap.position;
            let len = gap.length;

            gap.position += data.len();
            gap.length -= data.len();

            (pos, len)
        };

        if gap_length == 0 {
            self.gaps.remove(index);
        }

        let id = self.get_new_id();

        self.write_at(position, data, id);

        let entry = MeshEntry {
            id: id,
            position: position,
            length: data.len(),
        };
        let entry_index = self
            .data
            .iter()
            .position(|x| x.position > entry.position)
            .unwrap_or(self.data.len());
        self.data.insert(entry_index, entry);

        self.print_buffer_infos();

        Some(id)
    }

    fn try_merge_gap(&mut self, position: usize) {
        println!("Trying to merge gap of pos: {}.", position);
        let Some(mut current_index) = self.gaps.iter().position(|x| x.position == position) else {
            return;
        };

        if current_index > 0 {
            let prev = &self.gaps[current_index - 1];
            let curr = &self.gaps[current_index];
            if prev.position + prev.length == curr.position {
                self.gaps[current_index - 1].length += curr.length;
                self.gaps.remove(current_index);
                current_index -= 1;
            }
        }

        let current = &self.gaps[current_index];
        let curr_end = current.position + current.length;

        if current_index + 1 < self.gaps.len() {
            let next = &self.gaps[current_index + 1];
            if next.position == curr_end {
                self.gaps[current_index].length += next.length;
                self.gaps.remove(current_index + 1);
            }
        }
    }

    pub fn bytes_representation(bytes: u32) -> (u32, u32, u32, u32) {
        return (bytes, bytes / 1024, bytes / 1024 / 1024, bytes / 1024 / 1024 / 1024);
    }

    pub fn print_buffer_infos(&self) {
        let data_length = self.data.iter().fold(0, |acc, v| acc + v.length) as u32;
        let (len_b, len_kb, len_mb, _) = MeshManager::bytes_representation(data_length);
        let (cap_b, cap_kb, cap_mb, _) = MeshManager::bytes_representation(self.buffer.capacity());
        println!(
            "Mesh buffer\nLen: {:3}Mb/{:6}Kb/{:9}b\nCap: {:3}Mb/{:6}Kb/{:9}b",
            len_mb, len_kb, len_b, cap_mb, cap_kb, cap_b,
        );
    }

    pub fn free_data(&mut self, id: MeshId) {
        println!("Freeing data of mesh id: {}.", id);
        let Some(data_index) = self.data.iter().position(|x| x.id == id) else {
            return;
        };

        let entry = &self.data[data_index];
        let (position, length) = (entry.position, entry.length);

        let gap = Gap::new(position, length);

        self.insert_gap_after(gap, position);
        self.try_merge_gap(position);

        self.data.remove(data_index);
        self.write_operations.retain(|element| element.mesh_id != id);

        self.free_ids.push(id);

        self.print_buffer_infos();
    }
}
