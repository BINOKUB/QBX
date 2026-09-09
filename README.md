# QBX (Québec UNIX) — EXP

# QBX — Hardened Security Microkernel & Network Reconnaissance Appliance

**QBX** (Québec UNIX / Experimental) est un micro-noyau expérimental modulaire écrit de zéro en **Rust bare-metal (`no_std`)** pour l'architecture **x86_64**.

Conçu pour fonctionner sans dépendre des couches logicielles traditionnelles ou d'un noyau monolithique conventionnel, QBX priorise un modèle de sécurité étanche (RBAC tripartite), une mémoire volatile nettoyée cryptographiquement, et des capacités natives d'interception réseau directe sur le matériel (DMA / Promiscuous).

---

## 🛡️ Piliers Fondamentaux d'Architecture

### 1. Modèle de Sécurité RBAC Tripartite & Amorçage Clandestin
* **Hiérarchie à 3 paliers stricts :**
  * `>` **Opérateur :** Surveillance en lecture seule, audit matériel et télémétrie basique.
  * `#` **Administrateur :** Gestion des flux, configuration des interfaces et du système de fichiers.
  * `!` **Architecte :** Déboguage mémoire bas niveau, contrôle matériel direct et redéfinition des privilèges.
* **Cold-Provisioning (Amorçage à froid) :** Aucune empreinte ou condensat cryptographique n'est embarqué en dur dans la section `.rodata`. Le provisionnement se fait à la première initialisation (`initarch` / `initadm`).
* **Zéro allocation dynamique pour la crypto :** Hachage SHA-256 personnalisé calculé exclusivement sur la pile mémoire.
* **Résilience mémoire :** Vérifications cryptographiques en temps constant (`constant-time`) et nettoyage volatil forcé (`zeroize`) des tampons d'authentification après usage.

### 2. Sous-Système Réseau Bare-Metal & Surveillance Furtive
* **Pilote Intel e1000 / 82540EM :** Contrôle direct par registres MMIO mappés sur le bus PCI.
* **Capture DMA (RX Ring) :** Anneau circulaire matériel de 32 descripteurs avec tampons de 2048 octets pour la réception directe en RAM sans copie intermédiaire.
* **Mode Promiscuous Intégral :** Activation matérielle des drapeaux `UPE` (Unicast Promiscuous) et `MPE` (Multicast Promiscuous) dans le registre `RCTL` pour intercepter l'ensemble des trames du segment réseau local.
* **Sonde intégrée (`snf`) :** Analyseur d'en-têtes Ethernet en temps réel (détection automatique des protocoles ARP, IPv4, IPv6).

### 3. Gestionnaire Mémoire & Exécution
* **Pagination x86_64 à 4 niveaux :** Conversion et contrôle des adresses virtuelles et physiques matérielles.
* **Tas Dynamique (Heap) :** Gestion de l'allocateur global avec capacité d'extension dynamique.
* **Multitâche Préemptif :** Ordonnanceur cadencé par le contrôleur d'interruption PIT (IRQ 0) avec bascule de contexte sécurisée.
* **VFS Minimaliste :** Système de fichiers en mémoire vive avec support des descripteurs de flux et redirections (`>`, `>>`).

---

## 📋 Jeu de Commandes Résumé

| Commande | Palier Requis | Description |
| :--- | :---: | :--- |
| `pci` | Opérateur (`>`) | Scrutateur du bus PCI (énumération Mechanism 1, BAR0, IDs) |
| `net` | Opérateur (`>`) | Inspection de l'interface réseau, état du lien et adresse MAC |
| `snf` | Opérateur (`>`) | Sonde d'écoute réseau en mode Promiscuous |
| `afn` | Opérateur (`>`) | Consultation du journal des événements du noyau |
| `mnl` | Opérateur (`>`) | Manuel technique des commandes et structures |
| `su` | Opérateur (`>`) | Élévation sécurisée de privilèges vers Administrateur ou Architecte |
| `mem` | Opérateur (`>`) | Cartographie de la consommation du tas et des allocations |
| `clr` | Opérateur (`>`) | Purge du tampon d'affichage console |
| `initadm` | Opérateur (`>`) | Définition à froid de l'empreinte Administrateur |
| `initarch`| Administrateur (`#`) | Provisionnement du condensat Architecte maître |

---

## 🛠️ Environnement & Déploiement

### Dépendances de développement
* Rust Nightly (`rustup default nightly`)
* Cibles `x86_64-unknown-none`
* Cargo Bootimage (`cargo install bootimage`)
* Émulateur QEMU avec contrôleur réseau e1000

### Compilation et exécution locale
```bash
# Vérification statique du code
cargo check

# Génération de l'image disque amorçable
cargo bootimage

# Lancement de la machine avec interface réseau e1000
./run.sh

🎯 Feuille de Route Immédiate
[x] Scrutateur de bus matériel PCI

[x] Initialisation MMIO et anneau RX DMA (Intel e1000)

[x] Sonde d'interception réseau en mode Promiscuous (snf)

[ ] Anneau de transmission DMA (TX Ring) pour injection de trames brutes

[ ] Générateur et injecteur de requêtes ARP / sondes de découverte

[ ] Moteur de rendu graphique universel Framebuffer (UEFI GOP)

[ ] Déploiement sur matériel physique dédié (Appliances SFF multi-ports)
