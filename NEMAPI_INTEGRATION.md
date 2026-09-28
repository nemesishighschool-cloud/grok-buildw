# Intégration NemApi - Documentation Technique

## Architecture de l'intégration

### Schéma global
```
┌─────────────────────────────────────────────────────────────┐
│                    Grok Build (Client)                         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐  │
│  │  xai-grok-nemapi-provider (NOUVEAU)                      │  │
│  │  ├─ NemApiClient          - Client HTTP vers NemApi      │  │
│  │  ├─ NemApiSamplingClient  - Adaptateur pour le sampler    │  │
│  │  ├─ NemApiProvider        - Gestion des fournisseurs      │  │
│  │  ├─ NemApiResponseParser  - Parseur des réponses           │  │
│  │  └─ NemApiConfig          - Configuration                 │  │
│  └─────────────────────────────────────────────────────────┘  │
│                           │                                  │
│                           ▼                                  │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │  xai-grok-sampler (MODIFIÉ)                               │  │
│  │  ├─ SamplingClient        - Utilise NemApiClient          │  │
│  │  ├─ SamplerConfig         - Configuration mise à jour      │  │
│  │  └─ ...                                                 │  │
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
NemApi gère le **contexte côté serveur** via l'extension navigateur. Cela signifie :

1. **Première requête** : Envoie le prompt système + tous les messages + les outils
   ```json
   {
     "model": "gemini-chat",
     "messages": [
       {"role": "system", "content": "..."},
       {"role": "user", "content": "..."},
       {"role": "assistant", "content": "..."}
     ],
     "tools": [...],
     "stream": true,
     "provider": "gemini"
   }
   ```

2. **Requêtes suivantes** : Envoie **UNIQUEMENT** le dernier message utilisateur
   ```json
   {
     "model": "gemini-chat",
     "messages": [
       {"role": "user", "content": "nouveau message"}
     ],
     "stream": true,
     "provider": "gemini"
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
# Désactive l'API xAI officielle
export GROK_XAI_API_BASE_URL="http://127.0.0.1:8090/v1"

# Active NemApi
export GROK_NEMAPI_BASE_URL="http://127.0.0.1:8090/v1"

# Modèle par défaut
export GROK_DEFAULT_MODEL="gemini-chat"
```

### Configuration TOML
```toml
[endpoints]
xai_api_base_url = "http://127.0.0.1:8090/v1"
nemapi_base_url = "http://127.0.0.1:8090/v1"

[model]
default = "gemini-chat"

[nemapi]
default_provider = "gemini"
default_model = "gemini-chat"
stream_enabled = true
fresh_chat = true
premium_md = true
```

## Fournisseurs supportés

| Fournisseur | Modèle par défaut | Alias |
|-------------|------------------|-------|
| gemini | gemini-chat | gemini-2.5-flash, gemini-pro, flash |
| claude | claude-chat | claude-sonnet, claude-3-sonnet, sonnet |
| qwen | qwen-chat | qwen-plus, qwen2.5-plus, plus |
| deepseek | deepseek-chat | deepseek-coder, deepseek-v3, chat |
| chatgpt | gpt-chat | gpt-4, gpt-4o, gpt-5, o1, o3 |
| kimi | kimi-chat | kimi-k2, kimi-k3, moonshot |
| zai | glm-chat | glm-4, glm-5, chatglm |

## Modifications apportées

### 1. Module NemApi Provider
- ✅ Création de `xai-grok-nemapi-provider`
- ✅ Client HTTP dédié avec gestion de contexte
- ✅ Parseur robuste pour les imperfections NemApi
- ✅ Adaptateur compatible avec le sampler existant

### 2. Configuration
- ✅ Remplacement de `xai_api_base_url` par `nemapi_base_url`
- ✅ Configuration par défaut pointant vers NemApi
- ✅ Désactivation des autres endpoints xAI

### 3. Gestion des requêtes
- ✅ Première requête : contexte complet
- ✅ Requêtes suivantes : seul message utilisateur
- ✅ Suppression des requêtes anonymes

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
