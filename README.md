# AutoFish Local

[Pré-release Windows 0.1.2-alpha](https://github.com/LuisFPamplona/roblox-auto-fish/releases/tag/v0.1.2-alpha) · [Notas e limitações](docs/PRE_RELEASE_0.1.2.md).

Reconstrução independente do GPO Fishing Macro (Rust + Tauri + React/TypeScript). Licença MIT original preservada em LICENSE. Identificador próprio, dados próprios e atualização automática removida.

**Estado: versão experimental Windows x64 0.1.2-alpha com identidade visual própria, sem a imagem da logo original.** A correção de captura da versão 0.1.1 foi preservada. Consulte [docs/REDESIGN.md](docs/REDESIGN.md) para a interface e [docs/CAPTURE_FIX.md](docs/CAPTURE_FIX.md) para a validação no GPO. WINDOWS_VALIDATION_REPORT.md registra a primeira versão; SECURITY_AUDIT.md contém a revisão de segurança e docs/AUDITORIA.md a auditoria histórica do ZIP original.

## Compilar no Windows

Instale ferramentas oficiais:

1. Node.js LTS 24 ou superior: https://nodejs.org/
2. Rust stable (target x86_64-pc-windows-msvc): https://rustup.rs/
3. Visual Studio Build Tools com “Desktop development with C++”, MSVC e Windows 10/11 SDK: https://visualstudio.microsoft.com/visual-cpp-build-tools/
4. WebView2 runtime Microsoft, disponível normalmente no Windows 11. OCR local exige pacote de idioma compatível; a interface mostra disponibilidade.

Abra um Developer PowerShell do Visual Studio nesta pasta e execute:

```powershell
npm ci --ignore-scripts
npm audit --audit-level=high
npm run build
npm run test:security
cargo check --manifest-path src-tauri/Cargo.toml --locked
cargo test --manifest-path src-tauri/Cargo.toml --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
npm run app:build -- --no-bundle -- --locked
```

MakeItExe.bat executa essas verificações e interrompe no primeiro erro. O executável portátil fica em src-tauri/target/release/gpo-autofish.exe, ou em CARGO_TARGET_DIR/release quando essa variável estiver definida. Não instala o aplicativo, cria chaves ou usa atualizador. A instalação dos Build Tools exige autorização e pode precisar de administrador; executar o AutoFish não solicita elevação. Veja BUILD.md para pré-requisitos e reprodução. Lockfiles não asseguram binários byte a byte idênticos entre toolchains/SDKs.

Não ignore uma falha de Cargo ou de teste. Os testes Rust de segurança precisam ser executados para validar os guards; os testes Node não os substituem. A captura usa Windows Graphics Capture ligado à janela validada do Roblox. Falhas persistentes interrompem o macro e aparecem nos previews; não existe fallback para captura de monitor.

## Uso

Abra GPO no RobloxPlayerBeta.exe. Selecione regiões da barra e mensagens de fruta no editor e os pontos necessários. Coordenadas ficam relativas à área cliente; os cliques são limitados ao jogo. Comece com funcionalidades auxiliares desligadas e valide a área/preview antes de usar compra ou armazenamento.

F1 inicia/pausa, F2 edita regiões, F3 encerra, F4 alterna HUD (configuráveis). **Ctrl+Alt+F12 interrompe imediatamente novas entradas e libera as entradas geradas pelo macro; esse atalho é fixo.** A perda de foco pausa a automação. O editor e watchdog exigem retomada manual. OCR em andamento pode demorar a terminar, mas o gate de entrada é fechado antes de aguardar threads. Evite digitar/segurar teclas enquanto o macro automatiza o jogo.

Capturas ficam na memória e não são enviadas. Webhook vem desligado; informe seu próprio HTTPS discord.com/api/webhooks/... e ative explicitamente antes do teste. Notificações são limitadas a uma tentativa por cinco segundos; eventos em excesso são descartados. O token do webhook é mascarado na UI, porém fica em texto claro em settings.json. Presets não contêm esse segredo. Mais detalhes em docs/AUDITORIA.md.

Dados locais: %APPDATA%/autofish-local. Sem atualização automática: instale manualmente somente uma versão própria verificada. Nenhuma DLL injetada, alteração da memória do jogo, contorno de anticheat ou alteração do Defender faz parte deste projeto.

A auto-detecção oculta temporariamente apenas os controles do macro para examinar a área cliente do Roblox. O bot permanece pausado e os controles são restaurados ao terminar. O HUD padrão foi deslocado para baixo para deixar a área OCR livre.

O pacote de teste contém somente executável, licença, instruções e hash. Não inclui settings.json, webhooks, imagens capturadas, runtimes de auditoria ou dependências de desenvolvimento. WebView2 deve estar instalado. A distribuição portátil teve OCR sintético funcionando nesta máquina; a Microsoft documenta suporte desktop de Windows.Media.Ocr para apps com identidade de pacote/MSIX. Essa condição precisa ser resolvida antes de uma release pública.

O WebView2 gerou conexões HTTPS próprias durante o teste com webhooks desligados; seus payloads não foram analisados. Não considerar esta versão estritamente offline. Veja SECURITY_AUDIT.md para os destinos observados e a pendência de revisão do runtime antes de release pública.

Continuam pendentes: matriz de fullscreen/DPI/resoluções/GPUs, OCR dentro do GPO, pesca prolongada e compra/armazenamento. Os testes sintéticos não substituem esses cenários. O resultado da validação atual de captura e minigame está em docs/CAPTURE_FIX.md.
