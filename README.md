# AutoFish Local

Reconstrução independente do GPO Fishing Macro (Rust + Tauri + React/TypeScript). Licença MIT original preservada em LICENSE. Identificador próprio, dados próprios e atualização automática removida.

**Estado: código reconstruído; frontend compilado e verificado. Backend ainda precisa compilar e passar testes no Windows com MSVC. Não há instalador validado nem teste real no Roblox.** Leia docs/AUDITORIA.md para achados, linhas do fonte original, privacidade e limitações.

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
npm run app:build -- -- --locked
```

MakeItExe.bat faz essas verificações (incluindo testes de política da distribuição) e interrompe no primeiro erro. O instalador NSIS, se o build terminar, fica em src-tauri/target/release/bundle/nsis. Não cria chaves, não usa atualizador, não desativa proteção e não precisa executar como administrador. O build exige acesso aos registros oficiais e às ferramentas do bundler Tauri. Lockfiles fixam dependências, mas não asseguram instaladores byte a byte idênticos entre SDKs/toolchains; registre as versões usadas.

Não ignore uma falha de Cargo ou de teste. Os testes Rust de segurança precisam ser executados para validar os guards; os testes Node não os substituem. Se a captura GDI retornar preto, isso requer investigação de compatibilidade no Windows; não reative captura de monitor completo para contornar.

## Uso

Abra GPO no RobloxPlayerBeta.exe. Selecione regiões da barra e mensagens de fruta no editor e os pontos necessários. Coordenadas ficam relativas à área cliente; os cliques são limitados ao jogo. Comece com funcionalidades auxiliares desligadas e valide a área/preview antes de usar compra ou armazenamento.

F1 inicia/pausa, F2 edita regiões, F3 encerra, F4 alterna HUD (configuráveis). **Ctrl+Alt+F12 interrompe imediatamente novas entradas e libera as entradas geradas pelo macro; esse atalho é fixo.** A perda de foco pausa a automação. O editor e watchdog exigem retomada manual. OCR em andamento pode demorar a terminar, mas o gate de entrada é fechado antes de aguardar threads. Evite digitar/segurar teclas enquanto o macro automatiza o jogo.

Capturas ficam na memória e não são enviadas. Webhook vem desligado; informe seu próprio HTTPS discord.com/api/webhooks/... e ative explicitamente antes do teste. Notificações são limitadas a uma tentativa por cinco segundos; eventos em excesso são descartados. O token do webhook é mascarado na UI, porém fica em texto claro em settings.json. Presets não contêm esse segredo. Mais detalhes em docs/AUDITORIA.md.

Dados locais: %APPDATA%/autofish-local. Sem atualização automática: instale manualmente somente uma versão própria verificada. Nenhuma DLL injetada, alteração da memória do jogo, contorno de anticheat ou alteração do Defender faz parte deste projeto.

A auto-detecção oculta temporariamente apenas os controles do macro para examinar a área cliente do Roblox. O bot permanece pausado e os controles são restaurados ao terminar. O HUD padrão foi deslocado para baixo para deixar a área OCR livre.
