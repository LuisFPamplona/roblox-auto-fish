# Auditoria e reconstrução — AutoFish Local

Registro histórico da primeira reconstrução do ZIP. A etapa posterior de MSVC, compilação e execução Windows está documentada em `../SECURITY_AUDIT.md` e `../WINDOWS_VALIDATION_REPORT.md`. As limitações de build descritas aqui foram investigadas posteriormente; as evidências originais foram preservadas.

Data: 11/10/2026. Escopo: o ZIP fornecido, não a branch remota atual nem um instalador distribuído.
SHA-256 do ZIP original: `2815b68cafc96d0bf1bbc79c269af8e46e09c2920f6e41a556b537474d52a4d6`.
Inventário: 123 arquivos, 66 arquivos textuais, incluindo manifestos, lockfiles, scripts, todos os módulos Rust/TS e configurações. O inventário completo, hashes individuais, referências de rede, scripts npm transitivos e trechos de evidência estão em `original-inventory.json`.

**Não há executável original nem relatório detalhado das duas detecções. Não é possível atribuir essas detecções ao fonte, verificar equivalência fonte/binário ou declarar que o executável é seguro. Não foi encontrado indício direto de malware no código da aplicação. Dependências publicadas não foram auditadas linha por linha.** Nenhum executável do projeto original foi executado.

## Achados, classificação e resolução

As referências abaixo são linhas do ZIP ORIGINAL, preservadas pelo inventário; o código corrigido tem numeração diferente.

| Evidência original | Classificação | Risco e correção |
|---|---|---|
| `src-tauri/src/core/platform/windows/window.rs:112`, `src-tauri/src/app.rs:137` | Vulnerabilidade confirmada | Janela do macro contava como Roblox em primeiro plano e a perda de foco era retardada. Verificação fresca em cada entrada, apenas HWND do Roblox, pausa na perda de foco, liberação das teclas geradas e bloqueio de cliques sobre outras janelas. |
| `src-tauri/src/core/platform/windows/capture.rs:32`, `src-tauri/src/core/platform/windows/capture_dxgi.rs:136` | Funcionalidade legítima com risco associado | GDI lia desktop; DXGI copiava o monitor completo antes do recorte. DXGI removido. GDI usa DC da janela Roblox, recorte limitado, verificação de sobreposição antes/depois e recusa se o alvo mudar. |
| `src-tauri/src/commands.rs:276`, `src-tauri/src/commands.rs:74` | Vulnerabilidade confirmada | Coordenadas e parâmetros IPC sem validação central; max_dim=0 podia provocar divisão por zero, valores enormes pressionavam CPU/RAM e durações negativas podiam causar panic. Validação de regiões/pontos/durações/listas/taxas, limite de preview e gravação anterior à publicação das configurações. |
| `src-tauri/src/webhook.rs:169`, `src-tauri/src/webhook.rs:179` | Vulnerabilidade confirmada | Destino arbitrário, redirects habilitados, erros reqwest com URL secreta, fila sem limite e envio após desativar. Agora HTTPS apenas discord.com, caminho validado, redirects/proxy desabilitados, erros redigidos, fila 32, máximo uma tentativa/5s, rechecagem do opt-in e destino original da fila. |
| `src-tauri/src/commands.rs:401` | Vulnerabilidade confirmada | IPC aceitava URL/caminho/esquema arbitrário para opener. Agora só a URL exata do jogo; pasta local via comando sem argumento. Permissões frontend do opener removidas. |
| `src-tauri/tauri.conf.json:79` | Funcionalidade legítima com risco associado | Ausência de CSP aumentava impacto de comprometimento frontend. CSP local com rede frontend restrita a IPC, sem frames/objetos externos. |
| `src-tauri/tauri.conf.json:105`, `src/windows/Panel.tsx:53`, `src/pages/Settings.tsx:43` | Funcionalidade legítima com risco associado | Atualizador usava domínio/release e chave pública do autor original; check automático ignorava a preferência auto_update. A configuração tinha assinatura, portanto não é evidência de download sem validação nem de malware. Plugin, endpoints, chave, UI e artefatos updater removidos. |
| `MakeItExe.bat:44`, `MakeItExe.bat:65` | Funcionalidade legítima com risco associado | Build gerava chave privada sem senha e executava script PowerShell com ExecutionPolicy Bypass. Não alterava Defender/firewall. Build substituído por npm ci sem lifecycle scripts e Cargo locked; nenhuma geração de chaves. Script latest-json agora falha explicitamente. |
| `src-tauri/src/core/platform/windows/ocr.rs:34` | Comportamento suspeito que precisa de investigação | Engine WinRT compartilhado com unsafe Send/Sync e inicialização de apartment pouco robusta; problema de engenharia, sem evidência maliciosa. Engine/apartment por thread, tratamento de falhas e RoUninitialize no encerramento da thread. |
| `src-tauri/src/bot/mod.rs:197`, `src-tauri/src/windows.rs:353` | Vulnerabilidade confirmada | Recuperação podia reiniciar depois de uma parada concorrente; editor retomava automaticamente. Watchdog agora interrompe e exige início manual; editor permanece pausado. Lifecycle serializado. |
| `src-tauri/src/bot/trace.rs:114` | Funcionalidade legítima com risco associado | Trace grava números/tempos, não imagens. Continua opcional e desligado por padrão; não há upload desses arquivos. Removida a rotina que apagava recursivamente pastas antigas de frames na inicialização. |

## Investigação de comportamentos maliciosos

| Comportamento procurado | Resultado no código da aplicação |
|---|---|
| Senhas, .ROBLOSECURITY, cookies e sessões de navegador, tokens Discord | Nenhum acesso a perfis de navegador, bancos de cookies, credenciais ou tokens de contas encontrado. A URL de webhook do próprio usuário é um segredo diferente de token de conta. |
| Keylogging e hooks | Atalhos declarados via global-shortcut. Sem coleta de texto digitado; automação via SendInput. Nova tecla fixa Ctrl+Alt+F12 interrompe. |
| Imagens enviadas e webhooks ocultos | Sem upload de imagens nem webhook embutido encontrado. Reqwest enviava JSON ao destino configurado; foi restringido. |
| Execução remota, download de executáveis | Sem shell/process::Command na aplicação. Opener e updater eram as superfícies relacionadas; restringido/removido respectivamente. Build baixa ferramentas/dependências oficiais e o bundler pode obter ferramentas NSIS/WebView2 conforme Tauri. |
| Persistência, Defender, firewall, elevação | Não encontrados no fonte da aplicação/scripts. NSIS instala por usuário. Nenhuma tarefa agendada, serviço ou inicialização oculta foi adicionada. |
| Injeção de DLL, memória Roblox, anticheat, ocultação | Não encontrados nem adicionados. OpenProcess usa exclusivamente PROCESS_QUERY_LIMITED_INFORMATION para obter caminho do executável. |
| Arquivos pessoais e bibliotecas embutidas | Arquivos próprios em diretório de configuração; importação de JSON é selecionada pelo usuário. ZIP contém imagens/ícones, nenhum .exe/.dll/.sys pré-compilado. |
| Dependências comprometidas | Sem origem customizada nos lockfiles; npm e crates.io com integridade/checksums. Isso não comprova ausência de comprometimento na cadeia de fornecimento. |

**Falso positivo plausível, sem confirmação conclusiva:** captura GDI, SendInput, atalhos globais e instalador podem justificar heurísticas antivírus. Essa é uma hipótese; não conhecemos os nomes das detecções ou o binário examinado. Não houve tentativa de evasão.

## Rede e privacidade

Em execução, apenas notificações opcionais em `https://discord.com/api/webhooks/<id>/<token>` podem sair diretamente da aplicação. Campos: contagem de peixes/frutas, duração/taxa de sucesso, alerta de fruta lendária, nome/local reconhecidos de spawn, quantidade de iscas, tentativas e motivo interno de watchdog, e mensagem de teste. Não há imagens, cookies, senhas, tokens de contas ou texto OCR bruto no payload. Discord conhece IP e metadados normais da conexão. Uma requisição já iniciada pode terminar mesmo após desativar; eventos ainda na fila são descartados na rechecagem. O limite compartilhado vale também para testes; eventos excedentes são descartados, sem retry/spam.

A URL é mascarada na UI e não aparece nos erros HTTP. Continua armazenada em settings.json em texto claro no perfil local do usuário: não é um cofre de segredos. Presets exportados/salvos e presets carregados não transportam configuração de webhook; a importação v3 nunca ativa transmissão. O endereço de dados é novo: `%APPDATA%\autofish-local`, sem migração automática do original.

Capturas/preview/OCR ficam em RAM. Trace numérico opcional e logs/estatísticas/configuração são arquivos locais. O código original já não gravava capturas continuamente; não foi removido um upload de imagens inexistente. Logs de pesca podem conter texto reconhecido das mensagens do jogo. Não foi implementada retenção automática de todos os logs/traces: o usuário pode remover esses arquivos pelo diretório de dados.

Outros destinos: URL oficial da página GPO em roblox.com abre apenas após clique; localhost:1420 é o servidor Vite de desenvolvimento. Registros npmjs.org/crates.io e ferramentas oficiais são tráfego de build. GitHub Releases e convite Discord do autor original foram removidos da aplicação; aparecem somente na documentação forense do original.

## Dependências e componentes nativos

`npm audit` inicial encontrou source-map-js 1.2.1, GHSA-68fv-2mgg-jv7q; lockfile corrigido. Auditoria final npm: zero avisos conhecidos (`npm-audit.json`). Único lifecycle script sinalizado no lock npm original era fsevents, opcional macOS; npm ci --ignore-scripts impede sua execução. CLI Tauri, bindings nativos rolldown/oxc e Tailwind provêm de pacotes do registro oficial, com integridade. Não são executáveis reaproveitados do ZIP.

Consulta OSV inicial: 560 pacotes Rust de registry. Atualizador e 26 componentes órfãos removidos; rustls 0.23.44 corrigido para 0.23.45 conforme [RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285.html). Consulta final: 534 pacotes. [glib 0.18.5](https://rustsec.org/advisories/RUSTSEC-2024-0429.html) e [proc-macro-error](https://rustsec.org/advisories/RUSTSEC-2024-0370.html) permanecem no lock multiplataforma mas `cargo tree --target x86_64-pc-windows-msvc -i ...` confirmou ausência no alvo Windows. **Não considerar o Linux auditado/aprovado.** Cinco crates unic têm aviso de falta de manutenção, sem versão corrigida; são transitivas de urlpattern → tauri-utils, inclusive no Windows. Mantidas com risco documentado, sem substituir a stack inteira por um patch não verificado. [Aviso RustSec unic](https://rustsec.org/advisories/RUSTSEC-2025-0081.html). Não confundir avisos de manutenção com malware.

Rust/Cargo e rustfmt usados nesta revisão foram baixados de static.rust-lang.org, com SHA-256 da distribuição oficial verificado, sem instalação global. WebView2 é runtime Microsoft; OCR é componente Windows local; API Windows via crate windows do projeto Microsoft. Ícones/imagens vieram do ZIP, licença MIT original preservada. Build.rs local executa apenas tauri_build::build; build scripts transitivos do Cargo são código de terceiros e não receberam auditoria exaustiva. Não foram geradas chaves privadas.

## Limites e testes pendentes

Frontend compilado (TypeScript strict + Vite); verificações de política/distribuição Node são testes estáticos. Rustfmt confirma parsing, não checagem de tipos. `cargo check --locked` resolveu dependências mas parou com `link.exe not found`; **o backend não foi compilado nem os testes Rust executados**. Logs anexados em cargo-check.txt. Nenhum instalador foi produzido. Não há Roblox conectado para medir precisão OCR, estabilidade da pesca, CPU/RAM, resolução/DPI ou verificar compra/armazenamento.

Captura restrita GDI pode retornar quadro preto/recusar determinadas configurações DirectX, sobreposição de HUD/editor ou fullscreen exclusivo. Preview/auto-detecção exigem região desobstruída; minimize o painel se ele cobrir o jogo e use atalhos. O guard de z-order é conservador; mudanças transitórias entre verificações não são uma garantia formal de isolamento do compositor. Nome/classe/processo ajudam a selecionar Roblox, mas não verificam assinatura Authenticode do Roblox. Entrada e troca de foco não são operação atômica do Windows. Tais limitações exigem teste Windows real antes de chamar a versão de funcional/validada.

Funcionalidades preservadas no código: pesca/controle, parâmetros/paleta, regiões relativas, OCR de frutas/spawn e léxico, compra/seleção de iscas, armazenamento por cliques/teclas, zoom, presets, estatísticas, dashboard/HUD/tray/editor/preview/guia. Nenhuma funcionalidade invasiva adicionada. Watchdog preserva detecção e parada; recuperação automática foi suspensa por segurança. Alertas spawn continuam dependentes das opções já existentes do original. A UI permanece majoritariamente em inglês.

Referência de permissões: [Tauri Capabilities](https://v2.tauri.app/security/capabilities/). A aplicação removeu acesso frontend direto a updater/opener/atalhos; comandos próprios continuam na fronteira de confiança das janelas locais. CSP e validação central reduzem sua superfície, sem prometer que todo Tauri/transitivo foi formalmente verificado.

## Ajustes finais de integração

A visibilidade das janelas locais foi separada da autorização de entrada: controles próprios podem ficar visíveis quando o macro tem foco, sem permitir automação. Na auto-detecção solicitada pelo usuário, o bot pausa e as janelas próprias são ocultadas apenas durante a captura única da área cliente do Roblox; são restauradas após sucesso/erro. Isso evita interpretar o próprio painel como barra. O HUD novo fica inicialmente na parte inferior, fora da região OCR padrão. Preview que falha deixa de exibir a imagem anterior. Essas integrações passaram pelo parser Rust, porém aguardam validação nativa.
