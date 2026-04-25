//! Connexion réseau côté client.
//!
//! Ce module gère la connexion TCP côté client. Il encapsulate :
//! - La connexion TCP au serveur
//! - Le runtime Tokio pour les opérations async
//! - Le codec chiffré pour la communication
//! - Une file d'attente pour séquentialiser les envois
//!
//! ## Architecture de connexion
//!
//! ```text
//! ClientConnection
//! ├── runtime: Tokio runtime (pour les tâches async)
//! ├── stream: Arc<Mutex<TcpStream>> (connexion TCP)
//! ├── codec: EncryptedCodec (chiffrement)
//! ├── sender: mpsc::UnboundedSender<Paquet> (file d'attente d'envoi)
//! └── player_id: ID attribué par le serveur
//! ```
//!
//! ## File d'attente d'envoi (Résolution du problème de perte de paquets)
//!
//! Le problème initial : plusieurs tâches concurrentes essayaient d'écrire
//! sur le même TcpStream, causant des pertes de données.
//!
//! Solution : Utiliser un channel pour séquentialiser les envois.
//! - `send_packet()` met le paquet dans le channel (non-bloquant)
//! - Une seule tâche (`start_sender_task`) lit les paquets et les envoie
//! - Cela garantit l'ordre FIFO et élimine les écritures concurrentes

use shared::network::crypto::compute_shared_secret;
use shared::network::messages::{ContenuPaquet, Paquet, CURRENT_VERSION};
use shared::network::network_protocol::{create_codec, EncryptedCodec};
use shared::network::traits::PacketCodec;
use shared::{log_client, log_err_client};
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, Mutex};

/// Connexion au serveur.
///
/// Cette structure gère toute la communication réseau avec le serveur :
/// - Connexion TCP
/// - Handshake initial
/// - Envoi de paquets (via file d'attente)
///
/// # Utilisation
///
/// ```ignore
/// let mut conn = ClientConnection::new()?;
/// conn.connect("127.0.0.1:5000")?;
/// let (player_id, seed) = conn.perform_handshake("MonJoueur")?;
/// conn.send_packet(packet)?;
/// let response = conn.receive_packet()?;
/// ```
pub struct ClientConnection {
    /// Runtime Tokio pour exécuter les opérations async
    runtime: Option<Runtime>,
    /// Stream TCP protégé par un Mutex pour l'accès thread-safe
    stream: Option<Arc<Mutex<TcpStream>>>,
    /// Codec chiffré pour la communication
    codec: Arc<EncryptedCodec>,
    /// ID du joueur attribué par le serveur
    player_id: Option<u64>,
    /// Indique si la connexion est établie
    connected: bool,
    /// Channel pour l'envoi séquentialisé des paquets
    sender: Option<mpsc::UnboundedSender<Paquet>>,
}

impl ClientConnection {
    /// Crée une nouvelle connexion (non encore connectée).
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            runtime: None,
            stream: None,
            // Codec initial avec une clé nulle (sera remplacé après handshake)
            codec: Arc::new(create_codec([0u8; 32])),
            player_id: None,
            connected: false,
            sender: None,
        })
    }

    /// Retourne true si connecté au serveur.
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Retourne l'ID du joueur attribué par le serveur.
    pub fn player_id(&self) -> Option<u64> {
        self.player_id
    }

    /// Connecte au serveur TCP.
    ///
    /// # Arguments
    ///
    /// * `server_addr` - Adresse du serveur (ex: "127.0.0.1:5000")
    ///
    /// # Returns
    ///
    /// * `Ok(())` si la connexion a réussi
    /// * `Err(String)` si la connexion a échoué
    pub fn connect(&mut self, server_addr: &str) -> Result<(), String> {
        // Créer un nouveau runtime Tokio
        let runtime = Runtime::new().map_err(|e| e.to_string())?;

        // Bloquer pour établir la connexion TCP
        let stream =
            runtime.block_on(async { Ok::<TcpStream, String>(TcpStream::connect(server_addr).await.map_err(|e| e.to_string())?) })?;

        self.runtime = Some(runtime);
        self.stream = Some(Arc::new(Mutex::new(stream)));
        Ok(())
    }

    /// Effectue le handshake avec le serveur.
    ///
    /// # Flux de handshake
    ///
    /// 1. Recevoir le server_id (16 octets, non chiffré)
    /// 2. Calculer la clé partagée avec le server_id
    /// 3. Envoyer le paquet Handshake (chiffré)
    /// 4. Recevoir Confirmation (contient player_id)
    /// 5. Recevoir ServerSeed
    ///
    /// # Returns
    ///
    /// * `Ok((player_id, server_seed))` si le handshake a réussi
    /// * `Err(String)` si une erreur est survenue
    pub fn perform_handshake(&mut self, username: &str) -> Result<(u64, u32), String> {
        let runtime = self.runtime.as_ref().ok_or("No runtime")?;

        // Effectuer le handshake de manière bloquante
        let (player_id, server_seed, new_codec) = runtime.block_on(async {
            let mut stream = self.stream.as_mut().ok_or("Not connected")?.lock().await;

            // Étape 1: Recevoir le server_id
            let mut server_id_buf = [0u8; 16];
            stream.read_exact(&mut server_id_buf).await.map_err(|e| e.to_string())?;

            // Étape 2: Créer le codec avec la clé partagée
            let new_codec = Arc::new(create_codec(compute_shared_secret(&server_id_buf, b"server")));

            // Étape 3: Envoyer le paquet de handshake
            let packet = Paquet::new(
                shared::network::messages::TypePaquet::Handshake,
                ContenuPaquet::DonneesConnexion {
                    version: CURRENT_VERSION,
                    username: username.to_string(),
                },
            );

            new_codec.send_packet(&mut *stream, &packet).await.map_err(|e| e.to_string())?;

            // Étape 4: Recevoir la confirmation
            let confirmation_packet = new_codec.receive_packet(&mut *stream).await.map_err(|e| e.to_string())?;
            let player_id = match confirmation_packet.contenu {
                ContenuPaquet::Confirmation { player_id, .. } => player_id,
                _ => return Err("Unexpected packet".to_string()),
            };

            // Étape 5: Recevoir la seed du serveur
            let seed_packet = new_codec.receive_packet(&mut *stream).await.map_err(|e| e.to_string())?;
            let server_seed = match seed_packet.contenu {
                ContenuPaquet::ServerSeed { seed } => seed,
                _ => return Err("Expected server seed".to_string()),
            };
            log_client!("Server seed recue: {}", server_seed);

            Ok((player_id, server_seed as u32, new_codec))
        })?;

        // Mettre à jour l'état de la connexion
        self.codec = new_codec;
        self.player_id = Some(player_id);
        self.connected = true;

        // Démarrer la tâche d'envoi en arrière-plan
        self.start_sender_task();

        Ok((player_id, server_seed))
    }

    /// Démarre la tâche d'envoi en arrière-plan.
    ///
    /// Cette méthode crée un channel et lance une tâche Tokio qui :
    /// 1. Attend les paquets via le channel
    /// 2. Les envoie séquentiellement sur le stream TCP
    ///
    /// Cela résout le problème de concurrence d'écriture sur le stream.
    fn start_sender_task(&mut self) {
        let stream = self.stream.clone();
        let codec = self.codec.clone();

        // Créer un channel non-borné (unbounded)
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.sender = Some(tx);

        if let Some(stream) = stream {
            if let Some(runtime) = self.runtime.as_ref() {
                // Launcher une tâche qui traite les paquets dans l'ordre
                runtime.spawn(async move {
                    while let Some(packet) = rx.recv().await {
                        let guard = stream.lock().await;
                        let mut stream = guard;
                        // Envoyer le paquet (séquentiel, pas de concurrence)
                        if let Err(e) = codec.send_packet(&mut *stream, &packet).await {
                            log_err_client!("Erreur envoi packet: {}", e);
                        }
                    }
                });
            }
        }
    }

    /// Envoie un paquet au serveur.
    ///
    /// Cette méthode est non-bloquante : elle ajoute simplement le paquet
    /// dans la file d'attente. La tâche d'arrière-plan se charge de l'envoyer.
    ///
    /// # Avantages de cette approche
    ///
    /// - Non-bloquant : l'appel retourne immédiatement
    /// - Séquentiel : les paquets sont envoyés dans l'ordre
    /// - Sans concurrence : une seule tâche écrit sur le stream
    pub fn send_packet(&mut self, packet: Paquet) -> Result<(), String> {
        if !self.connected {
            return Ok(());
        }

        if let Some(sender) = &self.sender {
            // Ajouter le paquet dans la file d'attente (non-bloquant)
            sender.send(packet).map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Reçoit un paquet du serveur.
    ///
    /// Cette méthode est bloquante : elle attend jusqu'à ce qu'un paquet soit reçu.
    ///
    /// # Note
    ///
    /// Currently, cette méthode n'est pas utilisée par le client car le serveur
    /// pousse les mises à jour. Elle peut être utilisée pour une architecture
    /// où le client interroge le serveur.
    pub fn receive_packet(&mut self) -> Result<Paquet, String> {
        if !self.connected {
            return Err("Not connected".to_string());
        }

        let runtime = self.runtime.as_ref().ok_or("No runtime")?;
        let stream = self.stream.as_mut().ok_or("No stream")?;
        let codec = self.codec.clone();

        runtime.block_on(async {
            let mut stream = stream.lock().await;
            codec.receive_packet(&mut *stream).await.map_err(|e| e.to_string())
        })
    }
}
