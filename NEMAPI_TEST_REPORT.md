# Rapport de validation NemApi

## Résultats

| Vérification | Résultat | Détails |
|---|---|---|
| Catalogue NemApi | Réussi | `GET /v1/models` renvoie les sept identifiants canoniques : `deepseek-chat`, `qwen-chat`, `claude-chat`, `gemini-chat`, `gpt-chat`, `kimi-chat` et `glm-chat`. |
| Gemini sans compte | Partiel | L’extension Chromium a établi une conversation réelle avec `gemini-chat` et le premier échange a reçu une réponse. |
| Tâche complète de génération de code | Échec | Les essais de suivi ont dépassé le watchdog de l’extension (~230 s) ou ont renvoyé une erreur HTTP 500. Une réponse tardive de 193 caractères a été observée dans NemApi, mais elle n’a pas été remise au client Grok Build. La tâche n’est donc pas validée de bout en bout. |
| Tests Rust ciblés | Bloqué | La compilation de `xai-grok-shell` a d'abord révélé que `EndpointsConfig` ne déclarait plus `xai_api_base_url`, encore requis par les opérations xAI hors NemApi; le champ et sa valeur par défaut ont été restaurés. La relance a ensuite échoué faute d'espace disque (`No space left on device`, volume à 100 %), avant l'exécution des tests. |
| Formatage et whitespace des fichiers Rust modifiés | Réussi | `rustfmt --edition 2024 --check` sur les fichiers concernés et `git diff --check`. Le contrôle global signale aussi de nombreux fichiers préexistants hors périmètre. |

## Blocages observés dans NemApi

- Après le redémarrage de Chromium, NemApi conservait l’identifiant d’un ancien onglet Gemini; il a fallu sélectionner l’onglet courant dans `/extension/config`.
- Le sélecteur du bouton d’envoi Gemini ne trouvait pas toujours le bouton depuis la racine du compositeur. Une modification temporaire a été appliquée au clone pour poursuivre le test, puis restaurée; elle n’est pas incluse dans ce dépôt.
- Le watchdog déclarait certaines requêtes abandonnées avant l’arrivée d’une réponse tardive. Le client Grok Build recevait alors l’échec au lieu de cette réponse.

## Portée et conclusion

Le catalogue, le routage vers NemApi et l’adaptation du contexte sont configurés dans Grok Build. Le proxy local a bien exposé `gemini-chat` et une réponse réelle a été obtenue sans connexion utilisateur. Toutefois, faute de réponse de suivi remise à Grok Build, l’essai d’une tâche complète n’est pas concluant et l’intégration ne peut pas encore être déclarée validée de bout en bout.

Le test Gemini a été effectué via le proxy NemApi et l’extension Chromium; aucun exécutable autonome de Grok Build n’a été lancé dans cet environnement. Aucun accès à un compte DeepSeek n’a été tenté; la validation réelle de `deepseek-chat` via extension reste à faire.
