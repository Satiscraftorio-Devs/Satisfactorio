# Ascendustry

Jeu voxel multijoueur en Rust avec rendu wgpu.

```
Ascendustry/
├── project_core/  -- Types fondamentaux et structures partagées
├── game/          -- Monde, blocs, génération procédurale de terrain (noise)
├── engine/        -- Rendu GPU (wgpu), audio (kira), fenêtrage (winit)
├── network/       -- Réseau asynchrone (tokio), chiffrement AES-256-GCM
├── physics/       -- Physique et collisions (client & serveur)
├── client/        -- Client de jeu (rendu, audio, joueur)
├── server/        -- Serveur multijoueur (monde, connexions, logique)
└── launcher/      -- Lanceur graphique (eframe/egui)
```

## Dépendances principales

- **Rust** édition 2021
- **wgpu** 28 — rendu GPU (Vulkan/Metal/DX12)
- **winit** 0.30 — fenêtrage et entrées
- **tokio** — réseau asynchrone
- **kira** 0.12 — audio
- **serde** + **bincode** — sérialisation binaire
- **aes-gcm** + **sha2** — chiffrement AES-256-GCM
- **noise** 0.9 — génération procédurale de terrain
- **cgmath** + **bytemuck** — mathématiques 3D
- **rayon** — maillage parallèle des chunks
- **eframe** 0.27 — lanceur graphique
- **clap** — arguments en ligne de commande

## Utilisation

```bash
# Lancer le serveur
make server

# Lancer le client
make client

# Lancer le lanceur graphique
make launcher

# Tout lancer d'un coup (serveur arrière-plan + client)
make run

# Construire
make build

# Documentation
make doc
```

Les binaires acceptent `--address` / `-a` pour l'adresse de connexion (défaut : `127.0.0.1:42677`).

## Profiling

```bash
make client-profile     # Client avec flamegraph
make launcher-profile   # Lanceur avec flamegraph
```

## Contrôles

- `&` — mode fil de fer
- `é` — bordures de chunks

## Licence

Copyright StrachyDev & Theora59-dev 2025-2026. Tous droits réservés.
