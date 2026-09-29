# Intégration NemApi - Documentation Technique

## Architecture de l'intégration

### Schéma global
```
┌─────────────────────────────────────────────────────────────┐
│                    Grok Build (Client)                         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐  │
│  │  xai-grok-sampler (NemApi mode for canonical models)      │  │
│  │  ├─ Nouveau fil : prompt système + outils + message user  │  │
│  │  ├─ Suite/reprise : uniquement le dernier message user    │  │
│  │  └─ fresh_chat=true uniquement pour un nouveau fil       │  │
│  └─────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    NemApi Proxy                                │
│  (http://127.0.0.1:8090/v1)                                    │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │  - Gestion de contexte côté serveur                       │  │
│  │  - Routage vers les fournisseurs (Gemini, Claude, etc.)   │  │
│  │  - Automatisation navigateur via extension                │  │
│  └─────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    Fournisseurs IA                              │
│  (Gemini, Claude, Qwen, DeepSeek, ChatGPT, etc.)                 │
└─────────────────────────────────────────────────────────────┘
```

## Méthode NemApi - Gestion de contexte

### Principe fondamental
Le contexte conversationnel est conservé dans le fil du fournisseur piloté par
l'extension NemApi. L'historique affiché et persisté par Grok Build n'est pas
rejoué sur le réseau :

1. **Premier tour d'une nouvelle discussion** : envoie le prompt système, les
   définitions d'outils et le message utilisateur courant en une requête.
   `fresh_chat=true` demande à NemApi de démarrer un nouveau fil.
   ```json
   {
     "model": "gemini-chat",
     "messages": [
       {"role": "system", "content": "..."},
       {"role": "user", "content": "..."}
     ],
     "tools": [...],
     "stream": true,
     "provider": "gemini",
     "fresh_chat": true
   }
   ```

2. **Tours suivants et reprise avec `/resume`** : l'historique local reste
   visible, mais seule la nouvelle saisie utilisateur est envoyée.
   `fresh_chat=false` conserve le fil NemApi courant.
   ```json
   {
     "model": "gemini-chat",
     "messages": [
       {"role": "user", "content": "message à envoyer"}
     ],
     "stream": true,
     "provider": "gemini",
     "fresh_chat": false
   }
   ```

### Avantages
- ✅ Réduction du trafic réseau
- ✅ Meilleure gestion de la mémoire
- ✅ Contexte maintenu par le navigateur
- ✅ Pas besoin d'envoyer tout l'historique à chaque fois

## Configuration requise

### Variables d'environnement
```bash
# Endpoint NemApi (valeur par défaut)
export GROK_NEMAPI_BASE_URL="http://127.0.0.1:8090/v1"

# Modèle par défaut
export GROK_DEFAULT_MODEL="gemini-chat"
```

### Configuration TOML
```toml
[endpoints]
nemapi_base_url = "http://127.0.0.1:8090/v1"

[models]
default = "gemini-chat"
```

## Modèles affichés par `/model`

Le sélecteur affiche uniquement les sept noms canoniques NemApi :

`deepseek-chat`, `qwen-chat`, `claude-chat`, `gemini-chat`, `gpt-chat`,
`kimi-chat`, `glm-chat`.

## Modifications apportées

### 1. Client NemApi autonome
- ✅ Création de `xai-grok-nemapi-provider`
- ✅ Client HTTP dédié avec gestion de contexte (utilisable séparément)
- ✅ Parseur robuste pour les imperfections NemApi

### 2. Configuration
- ✅ Routage des sept modèles NemApi par défaut vers `nemapi_base_url`

### 3. Gestion des requêtes
- ✅ Premier tour neuf : prompt système, outils et message utilisateur
- ✅ Tours suivants / reprise : seul le nouveau message utilisateur; historique local non renvoyé
- ✅ `fresh_chat` actif au début d'une nouvelle discussion, désactivé ensuite
- ✅ Sélecteur limité aux sept noms canoniques

## Tests

### Test de base
```bash
# Démarrer NemApi
nemapi serve --port 8090

# Lancer Grok Build avec configuration NemApi
GROK_NEMAPI_BASE_URL="http://127.0.0.1:8090/v1" \
GROK_DEFAULT_MODEL="gemini-chat" \
grok-shell
```

### Test avec gemini-chat
```rust
use xai_grok_nemapi_provider::{NemApiClient, NemApiConfig};

let config = NemApiConfig::default()
    .with_default_provider("gemini")
    .with_default_model("gemini-chat");

let client = NemApiClient::new(config)?;
let response = client.send_chat_completion(request).await?;
```

## Problèmes connus et solutions

### 1. Format des réponses NemApi
**Problème** : Les réponses peuvent contenir des artefacts DOM (HTML, boutons, etc.)
**Solution** : Le parseur `NemApiResponseParser` nettoie automatiquement :
- Entités HTML
- Caractères de contrôle
- Éléments UI
- Blocs de réflexion

### 2. Streaming
**Problème** : Le streaming peut être intermittent
**Solution** : Gestion robuste des erreurs de stream avec reconnexion

### 3. Authentification
**Problème** : NemApi utilise son propre système d'authentification
**Solution** : Support des API keys au format `nemapi-token{random}`

## Prochaines étapes

1. ✅ Intégration du module NemApi
2. ✅ Configuration mise à jour
3. ⏳ Tests d'intégration
4. ⏳ Validation avec tâche complexe
5. ⏳ Documentation finale

## Références

- [NemApi GitHub](https://github.com/teteekoue/NemApi)
- [Documentation NemApi](https://github.com/teteekoue/NemApi/blob/main/README.md)
- [Grok Build](https://github.com/nemesishighschool-cloud/grok-buildw)
