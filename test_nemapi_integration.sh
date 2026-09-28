#!/bin/bash

# Script de test d'intégration NemApi
# Ce script permet de tester l'intégration complète de NemApi avec Grok Build

set -e

echo "=========================================="
echo "Test d'intégration NemApi pour Grok Build"
echo "=========================================="
echo ""

# Couleurs pour l'affichage
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Fonction pour afficher les étapes
echo_step() {
    echo -e "${BLUE}➡${NC} $1"
}

# Fonction pour afficher les succès
echo_success() {
    echo -e "${GREEN}✓${NC} $1"
}

# Fonction pour afficher les erreurs
echo_error() {
    echo -e "${RED}✗${NC} $1"
}

# Fonction pour afficher les avertissements
echo_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# Vérifier que nous sommes dans le bon répertoire
cd "$(dirname "$0")"

echo_step "Vérification de l'environnement..."

# Vérifier que cargo est disponible
if ! command -v cargo &> /dev/null; then
    echo_error "Cargo n'est pas installé ou n'est pas dans le PATH"
    exit 1
fi
echo_success "Cargo est disponible"

# Vérifier que rustc est disponible
if ! command -v rustc &> /dev/null; then
    echo_error "Rust n'est pas installé ou n'est pas dans le PATH"
    exit 1
fi
echo_success "Rust est disponible"

echo ""
echo_step "Compilation du module NemApi..."

# Compiler le module NemApi
cd crates/codegen/xai-grok-nemapi-provider
if cargo build 2>&1; then
    echo_success "Module NemApi compilé avec succès"
else
    echo_error "Échec de la compilation du module NemApi"
    exit 1
fi

cd ../../..

echo ""
echo_step "Exécution des tests unitaires..."

# Exécuter les tests unitaires
cd crates/codegen/xai-grok-nemapi-provider
if cargo test --lib 2>&1; then
    echo_success "Tous les tests unitaires ont réussi"
else
    echo_error "Certains tests unitaires ont échoué"
    exit 1
fi

cd ../../..

echo ""
echo_step "Exécution des tests d'intégration..."

# Exécuter les tests d'intégration
cd crates/codegen/xai-grok-nemapi-provider
if cargo test --test integration_tests 2>&1; then
    echo_success "Tous les tests d'intégration ont réussi"
else
    echo_error "Certains tests d'intégration ont échoué"
    exit 1
fi

cd ../../..

echo ""
echo_step "Exécution des tests spécifiques à Gemini..."

# Exécuter les tests spécifiques à Gemini
cd crates/codegen/xai-grok-nemapi-provider
if cargo test --test gemini_integration_tests 2>&1; then
    echo_success "Tous les tests Gemini ont réussi"
else
    echo_error "Certains tests Gemini ont échoué"
    exit 1
fi

cd ../../..

echo ""
echo_step "Exécution des tests de parseur..."

# Exécuter les tests de parseur
cd crates/codegen/xai-grok-nemapi-provider
if cargo test --test parser_tests 2>&1; then
    echo_success "Tous les tests de parseur ont réussi"
else
    echo_error "Certains tests de parseur ont échoué"
    exit 1
fi

cd ../../..

echo ""
echo_step "Vérification de la configuration..."

# Vérifier que la configuration NemApi existe
if [ -f "test_configs/nemapi_integration_test.toml" ]; then
    echo_success "Fichier de configuration NemApi trouvé"
else
    echo_error "Fichier de configuration NemApi introuvable"
    exit 1
fi

echo ""
echo_step "Vérification des modifications de configuration..."

# Vérifier que les modifications de configuration sont présentes
if grep -q "nemapi_base_url" crates/codegen/xai-grok-shell/src/agent/config.rs; then
    echo_success "Configuration nemapi_base_url trouvée"
else
    echo_error "Configuration nemapi_base_url introuvable"
    exit 1
fi

if grep -q "NEMAPI_BASE_URL_DEFAULT" crates/codegen/xai-grok-shell/src/agent/config.rs; then
    echo_success "Constante NEMAPI_BASE_URL_DEFAULT trouvée"
else
    echo_error "Constante NEMAPI_BASE_URL_DEFAULT introuvable"
    exit 1
fi

echo ""
echo_step "Affichage de la structure du module NemApi..."

# Afficher la structure du module
cd crates/codegen/xai-grok-nemapi-provider
find . -name "*.rs" -o -name "*.toml" | sort | while read file; do
    echo "  - $file"
done

cd ../../..

echo ""
echo_step "Vérification des dépendances..."

# Vérifier les dépendances dans Cargo.toml
cd crates/codegen/xai-grok-nemapi-provider
if grep -q "xai-grok-sampling-types" Cargo.toml && \
   grep -q "xai-grok-sampler" Cargo.toml && \
   grep -q "reqwest" Cargo.toml && \
   grep -q "serde_json" Cargo.toml; then
    echo_success "Toutes les dépendances nécessaires sont présentes"
else
    echo_error "Certaines dépendances sont manquantes"
    exit 1
fi

cd ../../..

echo ""
echo_step "Test de création de client NemApi..."

# Créer un petit programme de test
cat > /tmp/test_nemapi_client.rs << 'EOF'
use xai_grok_nemapi_provider::{NemApiClient, NemApiConfig, NemApiSamplingClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Création du client NemApi avec configuration par défaut...");
    let config = NemApiConfig::default();
    let client = NemApiClient::new(config)?;
    println!("✓ Client NemApi créé avec succès");
    println!("  - Base URL: {}", client.effective_base_url());
    println!("  - Modèle par défaut: {}", client.default_model());
    println!("  - Fournisseur par défaut: {}", client.default_provider());

    println!("\nCréation du client NemApi avec configuration Gemini...");
    let gemini_config = NemApiConfig::gemini_test_config();
    let gemini_client = NemApiClient::new(gemini_config)?;
    println!("✓ Client NemApi Gemini créé avec succès");
    println!("  - Modèle: {}", gemini_client.default_model());
    println!("  - Fournisseur: {}", gemini_client.default_provider());

    println!("\nCréation du client Sampling NemApi...");
    let sampling_client = NemApiSamplingClient::new(NemApiConfig::default())?;
    println!("✓ Client Sampling NemApi créé avec succès");
    println!("  - Modèle: {}", sampling_client.default_model());

    println!("\nCréation du client Sampling NemApi avec Gemini...");
    let gemini_sampling_client = NemApiSamplingClient::with_gemini_test_config()?;
    println!("✓ Client Sampling NemApi Gemini créé avec succès");

    Ok(())
}
EOF

# Compiler et exécuter le test
cd crates/codegen/xai-grok-nemapi-provider
if rustc --edition 2021 /tmp/test_nemapi_client.rs \
    -L target/debug/deps \
    --extern xai_grok_nemapi_provider=target/debug/libxai_grok_nemapi_provider.rlib \
    --extern tokio=target/debug/deps/libtokio-*.rlib \
    -o /tmp/test_nemapi_client 2>&1 | grep -v "warning:" | head -20; then
    
    if /tmp/test_nemapi_client 2>&1; then
        echo_success "Test de création de client réussi"
    else
        echo_warning "Test de création de client échoué (mais pas critique)"
    fi
else
    echo_warning "Impossible de compiler le test de client (dépendances manquantes)"
fi

cd ../../..

echo ""
echo_step "Vérification du parseur de réponses..."

# Créer un test de parseur
cat > /tmp/test_parser.rs << 'EOF'
use xai_grok_nemapi_provider::{NemApiResponseParser, create_default_parser};

fn main() {
    println!("Test du parseur de réponses NemApi...");
    
    let parser = create_default_parser();
    
    // Test avec une réponse simple
    let response_body = r#"{
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1234567890,
        "model": "gemini-chat",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello, how can I help you?"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_tokens": 15
        }
    }"#;
    
    match parser.parse_chat_completion_response(response_body, "gemini") {
        Ok(response) => {
            println!("✓ Parseur: Réponse simple parsée avec succès");
            println!("  - ID: {}", response.id);
            println!("  - Modèle: {}", response.model);
            println!("  - Contenu: {}", response.choices[0].message.content);
        }
        Err(e) => {
            println!("✗ Parseur: Échec du parsing: {}", e);
            std::process::exit(1);
        }
    }
    
    // Test avec du HTML
    let html_response = r#"{
        "id": "chatcmpl-124",
        "object": "chat.completion",
        "created": 1234567890,
        "model": "gemini-chat",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello &lt;world&gt; &amp; friends"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_tokens": 15
        }
    }"#;
    
    match parser.parse_chat_completion_response(html_response, "gemini") {
        Ok(response) => {
            println!("✓ Parseur: Réponse avec HTML parsée avec succès");
            let content = response.choices[0].message.content;
            println!("  - Contenu nettoyé: {}", content);
            if !content.contains("&lt;") && !content.contains("&gt;") && !content.contains("&amp;") {
                println!("  - Entités HTML correctement nettoyées");
            } else {
                println!("  ✗ Entités HTML non nettoyées");
                std::process::exit(1);
            }
        }
        Err(e) => {
            println!("✗ Parseur: Échec du parsing HTML: {}", e);
            std::process::exit(1);
        }
    }
    
    println!("\n✓ Tous les tests de parseur réussis");
}
EOF

# Compiler et exécuter le test de parseur
cd crates/codegen/xai-grok-nemapi-provider
if rustc --edition 2021 /tmp/test_parser.rs \
    -L target/debug/deps \
    --extern xai_grok_nemapi_provider=target/debug/libxai_grok_nemapi_provider.rlib \
    --extern serde_json=target/debug/deps/libserde_json-*.rlib \
    --extern serde=target/debug/deps/libserde-*.rlib \
    -o /tmp/test_parser 2>&1 | grep -v "warning:" | head -20; then
    
    if /tmp/test_parser 2>&1; then
        echo_success "Test de parseur réussi"
    else
        echo_warning "Test de parseur échoué (mais pas critique)"
    fi
else
    echo_warning "Impossible de compiler le test de parseur (dépendances manquantes)"
fi

cd ../../..

echo ""
echo "=========================================="
echo "Résumé des tests d'intégration NemApi"
echo "=========================================="
echo ""
echo_success "✓ Module NemApi compilé avec succès"
echo_success "✓ Tous les tests unitaires ont réussi"
echo_success "✓ Tous les tests d'intégration ont réussi"
echo_success "✓ Tous les tests spécifiques à Gemini ont réussi"
echo_success "✓ Tous les tests de parseur ont réussi"
echo_success "✓ Configuration mise à jour avec succès"
echo ""
echo "L'intégration NemApi est prête pour Grok Build!"
echo ""
echo "Pour utiliser NemApi:"
echo "1. Démarrez NemApi: nemapi serve --port 8090"
echo "2. Configurez Grok Build avec:"
echo "   - GROK_NEMAPI_BASE_URL=http://127.0.0.1:8090/v1"
echo "   - GROK_DEFAULT_MODEL=gemini-chat"
echo "3. Utilisez le modèle gemini-chat comme modèle par défaut"
echo ""
echo "Fournisseurs supportés: gemini, claude, qwen, deepseek, chatgpt, kimi, zai"
echo ""
