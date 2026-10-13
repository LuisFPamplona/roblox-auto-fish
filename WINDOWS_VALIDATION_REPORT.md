# Validação Windows — AutoFish Local 0.1.0-alpha

Data: 11/10/2026. Distribuição experimental x64 para teste local. **Captura e pesca reais no GPO NÃO TESTADAS. Não é uma release pública pronta.** Nenhum push ou release GitHub foi feito nesta etapa.

## Resultado

| Teste | Resultado | Evidência |
|---|---|---|
| Rust check | PASSOU | `cargo check --locked`; docs/windows-cargo-check.txt |
| Rust tests | PASSOU | 47 resultados positivos, zero falhas e dois ignorados na suíte padrão; docs/windows-cargo-tests.txt |
| Clippy | PASSOU | `cargo clippy --locked --all-targets -- -D warnings`; zero avisos; docs/windows-clippy.txt |
| Rust fmt | PASSOU | `cargo fmt --all -- --check`; saída 0; docs/windows-fmt.txt |
| Frontend build | PASSOU | TypeScript strict + Vite; build final também executado pelo Tauri; docs/windows-tauri-build.txt |
| Testes Node | PASSOU | Cinco testes estáticos; docs/windows-security-tests.txt |
| Tauri build | PASSOU | `tauri build --no-bundle -- --locked`; release x64; docs/windows-tauri-build.txt |
| Inicialização Windows | PASSOU | Janela real aberta; conexão, dashboard, Setup, Features e Settings inspecionados; docs/windows-ui-smoke.txt |
| Captura de tela real Roblox | NÃO TESTADO | Não havia Roblox em execução; não há evidência positiva de compatibilidade GDI |
| Rejeição de captura inválida | PASSOU | Teste Windows nativo de região vazia/excessiva, sem fallback de desktop |
| OCR local sintético | PASSOU | WinRT reconheceu “A Devil Fruit has spawned” em duas threads; docs/windows-ocr-test.txt |
| OCR dentro do GPO | NÃO TESTADO | Imagem sintética não representa o jogo |
| Reconhecimento de frutas/spawn | PASSOU | Testes de léxico, frases, falsos positivos e separação drop/spawn; não valida precisão OCR real |
| Pesca automática real | NÃO TESTADO | Sem jogo; barra/calibração/recolhimento reais pendentes |
| Ciclo de pesca sintético | PASSOU | Rastreamento, fim do ciclo, timeout e perda de foco contra cache antigo passaram |
| Atalho de emergência | PASSOU | Ctrl+Alt+F12 mudou Waiting for Roblox para Idle no executável; gate/release/disarm passaram em mocks |
| Emergência durante pesca real | NÃO TESTADO | Sem teclas/cliques nativos contra jogo neste teste |
| Encerramento | PASSOU | F3 encerrou o processo e os filhos WebView2 nas execuções observadas |
| Contador inicial | PASSOU | Teste de regressão passou; UI final não somou tempo ocioso anterior a F1 |
| HUD/editor sobre Roblox | NÃO TESTADO | Sem alvo; editor/preview reais, sobreposição e reposicionamento pendentes |
| Notificações Discord reais | NÃO TESTADO | Não autorizadas; nenhuma mensagem de teste enviada |
| Validação de webhook/default desligado | PASSOU | Testes Rust de destinos/opt-in/erro redigido e teste estático de distribuição |
| Análise de rede | PASSOU | Duas amostragens locais e netlog diagnóstico; evidências docs/windows-runtime-network*.json e windows-webview-network-hosts.json |
| Execução sem tráfego externo do runtime | FALHOU | HTTPS observado em filhos WebView2, mesmo sem webhook; conteúdo não inspecionado |
| Microsoft Defender | PASSOU | Verificação customizada com assinatura 1.459.669.0, retorno 0 e nenhuma ameaça encontrada; docs/windows-defender-scan.txt e windows-defender-status.json |
| Windows isolado | NÃO TESTADO | Windows Sandbox indisponível; execução feita na sessão local |
| NSIS | NÃO TESTADO | Distribuição portátil; nenhum instalador foi gerado ou executado |

Os 47 positivos da suíte padrão incluem um replay que retorna sem testar imagens porque a pasta de fixtures está vazia. Esse resultado **não comprova replay real**. Um ignorado é `dump_sim` diagnóstico; o outro é OCR, que foi executado separadamente e passou. Portanto há 48 resultados positivos contando a execução OCR separada, mas um deles continua sem cobertura por ausência de fixtures.

## Ambiente e correção de link.exe

Windows x64, kernel 10.0.26300.9457, DisplayVersion 26H2 (o registro retorna ProductName “Windows 10 Pro”; essa string não foi usada para inferir a edição comercial). Rust/Cargo/Clippy/rustfmt 1.99.0, host x86_64-pc-windows-msvc. WebView2 155.0.4283.45. Inventário em docs/windows-environment.json.

MSVC e SDK estavam ausentes, não apenas fora do PATH. Após autorização explícita, o instalador oficial Microsoft com assinatura válida instalou Build Tools 2022 17.14.41, MSVC 14.44.35207 e Windows SDK 10.0.26100.0; retorno 0, sem reinício. O Developer PowerShell oficial definiu PATH/INCLUDE/LIB/LIBPATH e disponibilizou cl/link/rc. Uma tentativa antecipada durante a instalação encontrou link.exe mas falhou com kernel32.lib ausente; a repetição após concluir o SDK passou.

Ferramentas Rust portáteis foram obtidas do servidor oficial com hashes verificados. Nenhum linker desconhecido ou alteração de proteção foi utilizado. Scripts locais de preparação estão fora da distribuição.

## Problemas encontrados e corrigidos

1. Tipo incorreto na comparação GetDIBits: i32/u32 impedia compilação. Comparação corrigida, quadros incompletos continuam recusados.
2. Access violation WinRT reproduzida na criação da engine após encerrar o MTA da verificação de disponibilidade. Retenção MTA com CoIncrementMTAUsage/CoDecrementMTAUsage e leases RAII corrigiu o teste; engine/apartamento permanecem por thread, sem unsafe Send/Sync. Criação da lease principal fora da UI preserva STA do WebView2.
3. Eventos nullable de leitura/janela eram incompatíveis com o enum internamente marcado. Agora valores/null são enviados diretamente; falhas de entrega são registradas.
4. Dois fixtures antigos de pesca não inicializavam MockWindow.find(); agora a checagem real de foco é mantida, com teste adicional contra cache desatualizado.
5. Teste visual detectou contagem do período anterior a Start. A sessão inicial começa pausada; teste de regressão e nova execução confirmaram a correção.
6. Versão alpha quebrava linha na navegação; versão curta legível, completa no tooltip/configurações.
7. Avisos Clippy e formatação corrigidos no código; script de build agora exige checks e gera portátil.

## Execução, arquivos e rede

Nenhuma automação começou ao abrir. Sem Roblox, os previews ficaram sem captura; F1 iniciou Waiting for Roblox, Ctrl+Alt+F12 parou e F3 encerrou. Esses testes não provam latência de emergência durante um OCR travado ou pesca real. Os testes sintéticos verificam bloqueio/liberação; comportamento real continua pendente.

Foram observados o executável e processos msedgewebview2.exe oficiais; não foram observados shells auxiliares da aplicação nas amostras. Arquivos próprios observados: stats.json e log em `%APPDATA%\autofish-local`. Cache WebView2 em `%LOCALAPPDATA%\local.autofish.desktop\EBWebView`. Não é monitoramento exaustivo de writes/processos transitórios. Esses dados não foram empacotados.

A primeira amostragem TCP não observou conexões; a segunda observou WebView2 com HTTPS para `2603:1063:e:115::365:7ea3` e `2603:1056:c03:2420::2`, porta 443. Zero TCP do processo AutoFish nas amostras. O netlog separado mostrou URLs locais Tauri/IPC e descoberta WPAD, sem explicar todos os sockets. Não houve captura do conteúdo TLS nem prova da finalidade desses destinos. **Não afirmar operação totalmente offline ou ausência universal de transmissão a partir disso.** Ver SECURITY_AUDIT.md para a documentação oficial e pendência de privacidade. Não foram alterados firewall, antivírus ou políticas Windows.

O Defender retornou ausência de ameaças; isso não certifica ausência de malware. O atualizador e a confiança no projeto original permanecem removidos. Binário experimental sem assinatura Authenticode comercial. Manifesto PE não solicita elevação; dependências importadas são bibliotecas Windows, sem DLL de auditoria ou do executável original.

## Artefatos finais

- Executável: `C:\dev\AutoFish\artifacts\AutoFish-Local-v0.1.0-alpha-win-x64\AutoFish-Local.exe`
- ZIP: `C:\dev\AutoFish\artifacts\AutoFish-Local-v0.1.0-alpha-win-x64.zip`
- Conteúdo: AutoFish-Local.exe, LICENSE, LEIA-ME.txt, SHA256SUMS.txt. Sem configuração, segredo, webhook real, logs, imagens capturadas, runtimes de desenvolvimento ou instalador.
- SHA-256 EXE: `2b3833f918d6a94ddba1b45e56c6695484457828b65fa0803e0b3586d1580542`
- SHA-256 ZIP: `7b78a9c8b1213eaa96720ed7247e0dd2e0813a351fdae509a1fe088321356ac9`
- ZIP: 5.154.189 bytes. Checksum externo ao lado do ZIP. Inventário dos fontes atual em docs/source-manifest.json; arquivos modificados/adicionados em docs/windows-modified-files.txt.

O binário entregue corresponde à última recompilação após as correções de sessão/UI. A revisão Git de origem permanece 8dc70c6; mudanças locais desta etapa não foram publicadas.

## Pendências antes de release pública

Validar GDI positivo em jogo real (janela/fullscreen/borderless, DPI, resoluções, monitores, obstrução/HUD, movimento e troca de resolução); investigar quadros pretos se ocorrerem, com alternativas oficiais restritas ao HWND. Validar pesca prolongada, precisão OCR/frutas/spawn e compra/armazenamento com pontos calibrados. Testar perda de foco, emergência e encerramento com entradas realmente seguradas. Executar em ambiente isolado e realizar análise de rede mais completa, esclarecendo/suprimindo por meios suportados o tráfego próprio do runtime sem desativar proteções.

A Microsoft limita suporte desktop de Windows.Media.Ocr a apps com identidade de pacote/MSIX. Sucesso local do portátil não resolve esse requisito de suporte para distribuição geral. Não instalar identidade/certificados nem transformar em release pública sem revisão e autorização específicas. Testes reais Discord dependem de autorização e webhook próprio. Pendências de dependências da auditoria anterior permanecem descritas em SECURITY_AUDIT.md.
