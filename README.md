# Elden Ring Death Counter 💀

Compteur de morts pour **Elden Ring**, écrit en **Rust**. Il détecte automatiquement les morts et le boss responsable par **OCR** sur la capture d'écran, puis enregistre les compteurs dans une base **SQLite** (`rusqlite`). L'interface est réalisée avec **iced**.

> Projet réalisé pour apprendre Rust. La conception (idées, architecture, enchaînement des traitements d'image, passage en OCR et sélection du meilleur résultat) vient de moi. Une IA m'a surtout aidé sur les parties que je ne maîtrisais pas, notamment le traitement d'image de base.

## ✨ Fonctionnalités

- **Détection automatique des morts** : l'OCR surveille l'écran (2 scans par seconde) et repère le texte de mort configuré.
- **Détection du boss** : après une mort, le nom du boss est lu dans une ou plusieurs zones de l'écran (plusieurs zones pour les combats contre plusieurs boss, leurs noms sont alors regroupés dans un seul compteur : `Boss A - Boss B`).
- **Compteurs par boss** :
  - correspondance exacte avec un compteur existant ;
  - correspondance approximative (distance de Levenshtein normalisée via `strsim`, seuil de 80 %) pour tolérer les erreurs d'OCR ;
  - création automatique d'un compteur si le boss est inconnu.
- **Compteurs globaux** : morts totales et nombre de combats contre des boss.
- **Gestion manuelle** : `+`, `-`, remise à zéro, suppression, renommage d'un compteur en cliquant sur son titre.
- **Réorganisation par glisser-déposer** ; un boss nouvellement créé ou mis à jour remonte en tête de liste.
- **Activation par compteur** : chaque compteur peut être activé ou désactivé.
- **Raccourci clavier global** (`Alt` + `+`, Windows) pour incrémenter les compteurs activés sans quitter le jeu.
- **Ajout de compteurs personnalisés** via un écran dédié.
- **Persistance SQLite** (`rusqlite`, SQLite embarqué) avec **sauvegarde automatique** toutes les 10 secondes dès qu'il y a une modification.
- **Interface bilingue** (français / anglais). La langue ne change que les textes de l'application, pas le texte de mort à détecter.
- **Activation/désactivation de l'OCR** en un clic, avec statut en temps réel (démarrage, recherche d'une mort, recherche du nom du boss, pause).

## 🖥️ Interface

| Écran | Rôle |
|---|---|
| **Principal** | Bascule de l'OCR et statut, liste des compteurs, accès aux autres écrans |
| **Ajout d'un compteur** | Création d'un compteur (le titre ne peut pas être vide) |
| **Paramètres** | Jeu, langue, écran à capturer, texte de mort à détecter |

<!-- À compléter : captures d'écran -->

## 🔍 Fonctionnement de l'OCR

L'OCR est fourni par [`uni-ocr`](https://crates.io/crates/uni-ocr) avec le mode `OcrProvider::Auto`, qui choisit le moteur selon le système : sur **Windows**, c'est l'OCR intégré à Windows. Le projet n'a été **testé que sur Windows**.

### Détection de la mort

1. **Capture** de l'écran choisi dans les paramètres (`xcap`), exécutée hors du thread de l'interface.
2. **Découpe** de la zone de mort, définie par jeu sous forme de positions relatives à l'écran.
3. **Pré-filtre** : si moins de 1 % des pixels de la zone sont à dominante rouge, l'OCR n'est pas lancé. La boucle de surveillance reste ainsi légère.
4. **Prétraitement V1** : extraction du canal rouge, correction gamma 0,4, redimensionnement à 120 px de hauteur, puis OCR.
5. **Prétraitement V2** (si V1 échoue) : gamma 0,3, contraste ×2, redimensionnement à 120 px, puis OCR.
6. **Validation** : la mort est confirmée si le texte lu contient le texte de mort configuré (après nettoyage : majuscules, espaces et accents courants retirés), ou si la similarité Jaro-Winkler dépasse 80 %.

### Détection du nom du boss

Lancée seulement après une mort, pour chaque zone de boss, l'une après l'autre :

1. **7 versions** de l'image sont générées en parallèle (`rayon`) : niveaux de gris, gamma entre 0,20 et 0,40, avec ou sans contraste, puis agrandissement ×4 (Lanczos3).
2. Chaque version passe dans l'OCR et le texte est nettoyé (lettres, espaces, apostrophes, tirets).
3. Chaque résultat reçoit un **score de qualité** : longueur, nombre de mots, ratio de lettres, pénalités pour les caractères suspects, les chiffres, les répétitions et autres artefacts d'OCR.
4. Un **bonus** est ajouté quand plusieurs versions donnent le même texte.
5. Le meilleur score l'emporte. Si une zone ne donne rien d'exploitable, la recherche s'arrête.

### ⚠️ Limites connues

- La **détection de la mort** est plutôt fiable.
- La **détection du nom des boss est loin d'être parfaite** : selon la police, le fond de l'écran, la résolution ou la langue du jeu, le texte peut être mal lu, incomplet ou non détecté.
- La comparaison floue (seuil de 80 %) limite les doublons dus aux petites erreurs, mais elle ne les élimine pas : il peut arriver qu'un compteur soit créé avec un nom approximatif, ou qu'une mort soit attribuée au mauvais boss.
- Les zones surveillées sont fixes : elles ne sont pas encore modifiables depuis l'application.
- Il faut donc relire de temps en temps la liste et corriger les compteurs à la main si besoin (renommage, `+` / `-`, suppression).

### Boucle de surveillance

- Un scan toutes les 500 ms tant qu'aucune mort n'est détectée.
- À la détection d'une mort : le statut passe immédiatement à « recherche du boss », puis l'OCR du boss tourne dans une tâche `tokio` séparée pour ne pas bloquer l'interface.
- Une pause de 10 s suit une détection, pour ne pas compter deux fois le même écran de mort (anti-doublon de 5 s en plus).

## 🧱 Architecture

```
src/
├── hotkey/                # Raccourci clavier global (Windows)
├── i18n/
│   ├── language.rs        # Langues disponibles (Français, English)
│   └── translations.rs    # Clés de traduction par catégorie
├── ocr/
│   └── ocr.rs             # Détection de la mort et lecture du nom du boss
├── screens/
│   ├── main_screen.rs     # Écran principal (liste + OCR)
│   ├── add_recorder_screen.rs
│   ├── settings_screen.rs
│   └── components/
│       ├── list.rs        # Liste des compteurs (glisser-déposer, sauvegarde auto)
│       └── ocr.rs         # Composant OCR (statut, toggle, subscription)
├── structs/               # Recorder, Storage, Settings, Screen, GameConfig, CropPosition...
├── style/                 # Styles des conteneurs et boutons
└── utils/
    ├── app_worker.rs      # Workers : OCR et raccourci clavier (subscriptions iced)
    ├── image_processing.rs
    └── screen_capture.rs
```

L'application suit l'architecture **Elm** d'iced (`State` / `Message` / `update` / `view`). Chaque écran et composant possède ses propres messages, remontés au parent avec `.map(...)`. L'OCR et le raccourci clavier tournent dans des **subscriptions** iced qui envoient leurs événements à l'interface.

## ⚙️ Configuration

Dans l'écran **Paramètres** :

- **Jeu** : jeu surveillé (pour l'instant, uniquement **Elden Ring**) ;
- **Langue** : langue de l'interface (indépendante du jeu et du texte de mort) ;
- **Écran** : moniteur à capturer ;
- **Texte de mort** : texte à détecter à l'écran, à saisir **en majuscules** (la comparaison se fait sur le texte lu en majuscules).

Pense à lancer Elden Ring en **fenêtré sans bordure** pour que la capture fonctionne.

## 🛠️ Technologies

- [Rust](https://www.rust-lang.org/)
- [iced](https://iced.rs/) 0.14 + [iced_aw](https://github.com/iced-rs/iced_aw) (spinner)
- [tokio](https://tokio.rs/) pour l'asynchrone
- [rusqlite](https://github.com/rusqlite/rusqlite) pour la persistance
- [uni-ocr](https://crates.io/crates/uni-ocr) pour l'OCR
- [xcap](https://crates.io/crates/xcap) pour la capture d'écran
- [image](https://crates.io/crates/image) et [imageproc](https://crates.io/crates/imageproc) pour le traitement d'image
- [strsim](https://crates.io/crates/strsim) : Levenshtein normalisé (noms de boss) et Jaro-Winkler (texte de mort)
- [rayon](https://crates.io/crates/rayon) pour paralléliser les prétraitements
- [serde](https://serde.rs/), [uuid](https://crates.io/crates/uuid), [directories](https://crates.io/crates/directories), [anyhow](https://crates.io/crates/anyhow)

## 🗺️ Pistes d'amélioration

- Améliorer la fiabilité de la détection du nom des boss
- Permettre de **modifier les zones surveillées** (mort et boss) depuis les paramètres, pour s'adapter à chaque jeu et à chaque résolution
- Statistiques et historique des morts
- Export des données
- Raccourci clavier multiplateforme (actuellement Windows uniquement)
- Autres jeux (le choix du jeu est déjà dans les paramètres, mais seul Elden Ring est pris en charge pour l'instant)

## 📄 Licence

[MIT](https://opensource.org/license/mit)
