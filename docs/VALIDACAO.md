# Validação — 11/10/2026

## Executado neste ambiente

- ZIP inventariado antes de alterar/executar o projeto; hashes e referências originais em original-inventory.json.
- npm ci --ignore-scripts com pacotes oficiais e integridade do lockfile.
- npm audit: source-map-js corrigido; relatório final sem avisos conhecidos.
- OSV: rustls corrigido para 0.23.45; avisos remanescentes descritos na auditoria.
- Cargo update --workspace removeu updater/órfãos; versão rustls atualizada pontualmente, demais versões preservadas.
- TypeScript strict + Vite build passaram; ver logs frontend-build.txt.
- Testes Node de política/distribuição: resultado em security-tests.txt. Estes testes não executam o backend.
- Rustfmt executado com sucesso nos módulos e novos testes Rust, depois verificado com --check. Isso verifica parsing/formatação, não tipos.
- Cargo check --locked tentou o backend e falhou por ausência de link.exe/MSVC. Testes Rust, release e NSIS não executados. Log cargo-check.txt.
- CLI Tauri build --help confirmou forwarding de --locked pelo argumento final.

## Obrigatório antes de distribuir

1. Compilar e executar cargo test --locked no Windows com MSVC/SDK, incluindo tests/security.rs, tests/replay.rs e os testes unitários originais/novos. Corrigir qualquer erro e repetir.
2. Iniciar com webhooks e auxiliares desligados; verificar que nada automatiza antes de F1.
3. Validar barra, rastreamento e recolhimento em pelo menos 720p/1080p/1440p, escala DPI 100/125/150%, monitores com origem negativa e modo janela/borderless. Medir CPU/RAM, duração de sessão e taxa de captura.
4. Cobrir a região do Roblox com outro app e verificar recusa de captura; trocar foco durante clique, tecla segurada, compra e recolhimento. Confirmar ausência de novas entradas no outro app e liberação das entradas geradas. Validar falso alvo de título Roblox e minimização.
5. Durante cada operação, usar Ctrl+Alt+F12 e botão Stop. Verificar que watchdog/editor não reativam automação. Reservar a tecla de emergência; falha no registro impede startup.
6. Testar OCR local com idioma disponível/ausente e texto inválido. Verificar que regiões/pontos inválidos importados são recusados e que preview max_dim=0 não causa panic.
7. Validar iscas/compra/armazenamento apenas após os essenciais funcionarem; conferir pontos antes de operações que gastem moeda/alterem inventário.
8. Com webhook próprio de teste, confirmar campos autorizados, ausência de imagens/tokens nos logs, limite de 5s, redirecionamentos recusados, desativação e mudança de URL descartando fila anterior. Sem esse opt-in, nenhuma notificação deve sair.
9. Compilar release/NSIS, registrar versões de Node/Rust/MSVC/SDK, SHA-256 do instalador e revisão independente. Não atribuir segurança total ou equivalência ao executável original.

Não foi observado o Roblox neste ambiente. Esta lista é trabalho pendente, não evidência de sucesso.
