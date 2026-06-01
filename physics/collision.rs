use crate::aabb::AABB;
use crate::body::PhysicsBody;
use crate::collision_world::CollisionWorld;
use cgmath::{Point3, Vector3};
use game::constants::{COLLISION_EPSILON, PLAYER_HEIGHT, PLAYER_WIDTH};

fn aabb_at_feet(feet: &Point3<f32>) -> AABB {
    let half_width = PLAYER_WIDTH / 2.0;
    let half_height = PLAYER_HEIGHT / 2.0;
    let center = Point3::new(feet.x, feet.y + half_height, feet.z);
    AABB::new_sized(center, Vector3::new(half_width, half_height, half_width))
}

pub fn get_colliding_blocks(world: &impl CollisionWorld, aabb: &AABB) -> Vec<(i32, i32, i32)> {
    if world.is_empty() {
        return Vec::new();
    }

    let min_x = aabb.min.x.floor() as i32;
    let max_x = (aabb.max.x - COLLISION_EPSILON).floor() as i32;
    let min_y = aabb.min.y.floor() as i32;
    let max_y = (aabb.max.y - COLLISION_EPSILON).floor() as i32;
    let min_z = aabb.min.z.floor() as i32;
    let max_z = (aabb.max.z - COLLISION_EPSILON).floor() as i32;

    if min_x > max_x || min_y > max_y || min_z > max_z {
        return Vec::new();
    }

    let capacity = ((max_x - min_x) * (max_y - min_y) * (max_z - min_z)) as usize;
    let mut blocks = Vec::with_capacity(capacity);
    for x in min_x..=max_x {
        for y in min_y..=max_y {
            for z in min_z..=max_z {
                if world.is_block_solid(x, y, z) {
                    blocks.push((x, y, z));
                }
            }
        }
    }
    blocks
}

pub fn resolve_collision(world: &impl CollisionWorld, body: &mut PhysicsBody, dt: f32, position: &mut Point3<f32>) {
    let mut velocity = *body.velocity_mut().current();
    velocity.y += body.gravity * dt;

    position.y += velocity.y * dt;
    if velocity.y > 0.0 {
        let aabb = aabb_at_feet(position);
        let nearest = get_colliding_blocks(world, &aabb)
            .iter()
            .filter(|&(_, by, _)| (*by as f32) > position.y - PLAYER_HEIGHT)
            .map(|&(_, by, _)| by)
            .min();
        if let Some(by) = nearest {
            position.y = by as f32 - PLAYER_HEIGHT - COLLISION_EPSILON;
            velocity.y = 0.0;
        }
    } else if velocity.y < 0.0 {
        let aabb = aabb_at_feet(position);
        let nearest = get_colliding_blocks(world, &aabb)
            .iter()
            .filter(|&(_, by, _)| (*by as f32) < position.y)
            .map(|&(_, by, _)| by)
            .max();
        if let Some(by) = nearest {
            position.y = by as f32 + 1.0 + COLLISION_EPSILON;
            body.on_ground = true;
            velocity.y = 0.0;
        }
    }

    position.x += velocity.x * dt;
    {
        let aabb = aabb_at_feet(position);
        let blocks = get_colliding_blocks(world, &aabb);
        if velocity.x > 0.0 {
            if let Some(bx) = blocks
                .iter()
                .filter(|&&(bx, _, _)| (bx as f32) >= position.x)
                .map(|&(bx, _, _)| bx)
                .max()
            {
                position.x = bx as f32 - PLAYER_WIDTH / 2.0 - COLLISION_EPSILON;
                velocity.x = 0.0;
            }
        } else if velocity.x < 0.0 {
            if let Some(bx) = blocks
                .iter()
                .filter(|&&(bx, _, _)| (bx as f32) + 1.0 <= position.x)
                .map(|&(bx, _, _)| bx)
                .min()
            {
                position.x = bx as f32 + 1.0 + PLAYER_WIDTH / 2.0 + COLLISION_EPSILON;
                velocity.x = 0.0;
            }
        }
    }

    position.z += velocity.z * dt;
    {
        let aabb = aabb_at_feet(position);
        let blocks = get_colliding_blocks(world, &aabb);
        if velocity.z > 0.0 {
            if let Some(bz) = blocks
                .iter()
                .filter(|&&(_, _, bz)| (bz as f32) >= position.z)
                .map(|&(_, _, bz)| bz)
                .max()
            {
                position.z = bz as f32 - PLAYER_WIDTH / 2.0 - COLLISION_EPSILON;
                velocity.z = 0.0;
            }
        } else if velocity.z < 0.0 {
            if let Some(bz) = blocks
                .iter()
                .filter(|&&(_, _, bz)| (bz as f32) + 1.0 <= position.z)
                .map(|&(_, _, bz)| bz)
                .min()
            {
                position.z = bz as f32 + 1.0 + PLAYER_WIDTH / 2.0 + COLLISION_EPSILON;
                velocity.z = 0.0;
            }
        }
    }

    body.velocity.update(velocity);
}
