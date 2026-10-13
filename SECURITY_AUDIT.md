# Revisão de segurança da versão experimental Windows

Atualização 12/10/2026, versão 0.1.1-alpha: a captura ativa passou de GDI para
Windows Graphics Capture associado exclusivamente ao HWND validado do Roblox.
Não se ignora uma lista de janelas por nome nem se captura o monitor para
contornar sobreposição. Texturas são recortadas dentro da área cliente, com
limites de recurso e revalidação de identidade/geometria/foco. Objetos COM/D3D
permanecem em um worker; sessões ociosas e encerradas são liberadas. Falhas
persistentes desarmam entradas e param o bot. Captura positiva no GPO foi
observada nesta máquina; resultados e limites em [docs/CAPTURE_FIX.md](docs/CAPTURE_FIX.md).
As seções seguintes registram a validação histórica 0.1.0-alpha, inclusive as
pendências de OCR portátil e tráfego próprio do WebView2 ainda não resolvidas.

Data: 11/10/2026. Base: reconstrução já auditada do ZIP original; histórico detalhado, hashes e achados em `docs/AUDITORIA.md`. Esta revisão continua a mesma base. Não é auditoria exaustiva de todos os crates, nem certificação de ausência de malware.

## Ambiente e cadeia de compilação

O erro original `link.exe not found` decorreu da ausência de MSVC e Windows SDK. Após autorização explícita do proprietário, o instalador oficial `https://aka.ms/vs/17/release/vs_BuildTools.exe`, assinatura Authenticode válida Microsoft Corporation, instalou Build Tools 2022 17.14.41, MSVC 14.44.35207 e SDK 10.0.26100.0 em `C:\BuildTools` e Windows Kits. Código de saída 0; sem reinício solicitado. SHA-256 do bootstrapper: `985969f472caad75d993a5cb4c35a6a4271460cc12b343e2433b994d173aa990`.

Rust/Cargo/Clippy/rustfmt 1.99.0 x86_64-pc-windows-msvc vieram de `static.rust-lang.org`, com hashes do manifesto oficial verificados. Não houve substituição por linker desconhecido. A sessão importa o Developer PowerShell oficial para definir PATH, INCLUDE, LIB e LIBPATH; ter somente o linker no PATH não basta.

## Correções desta etapa

- Retorno GDI `GetDIBits` comparado como `i32`, preservando a recusa de quadros incompletos.
- Eventos nullable `bot:reading` e `roblox:changed` enviados diretamente como valor/null, sem enum internamente marcado incompatível com `None`. Falhas de entrega deixam de ser ignoradas.
- Formatação de teste replay antigo corrigida.
- Testes Windows adicionais de região inválida e OCR WinRT sintético em threads independentes. OCR sintético não comprova OCR real no GPO.
- Crash WinRT `0xc0000005` reproduzido: nova ativação da engine após a thread de disponibilidade encerrar o último MTA. Uma lease RAII `CoIncrementMTAUsage`/`CoDecrementMTAUsage` mantém o suporte MTA entre workers, com apartamentos/engines ainda locais a cada thread. A lease principal é criada fora da thread UI para preservar o STA do WebView2; cada engine mantém sua lease até liberar os objetos COM. Teste sintético de duas threads reconheceu a frase após a correção. Log de reprodução em `docs/windows-ocr-diagnostic.txt`; resultado corrigido em `docs/windows-ocr-test.txt`.
- Fixtures dos ciclos de pesca passaram a inicializar a janela mock real consultada pelo guard, preservando a checagem de foco. Teste adicional verifica cache de janela desatualizado e liberação de mouse na perda de foco.
- Avisos Clippy corrigidos no código e testes, sem atributos de supressão. Identificação da distribuição alterada para `0.1.0-alpha` sem atualizar versões de crates/npm transitivos.
- Teste visual do executável revelou soma indevida do tempo ocioso anterior ao primeiro Start; a sessão inicial agora começa pausada e possui teste de regressão. Exibição lateral da versão encurtada com versão completa em tooltip/configurações.

## Controles preservados

Captura por HWND identificado por classe/título/executável RobloxPlayerBeta.exe, sem captura de desktop como fallback. Região deve caber na área cliente; há limites de memória/dimensões, verificações de sobreposição e geometria antes/depois. Objetos GDI são liberados e bitmap é deselecionado antes de GetDIBits. Não há prova de compatibilidade com renderização Roblox, fullscreen/DPI ou todas as corridas de movimento/foco até testes reais.

Entrada é habilitada por comando do usuário, passa pelo gate de foco e coordenadas, e só libera entradas previamente geradas. Cliques/roda verificam propriedade do ponto. Ctrl+Alt+F12 é fixo e reservado. Watchdog exige retomada manual. Não há DLL injection, modificação/leitura da memória do jogo, manipulação de anticheat ou ações para evitar detecção.

Webhooks continuam desativados por padrão. Somente HTTPS discord.com/api/webhooks/id/token; sem redirecionamento ou proxy, fila limitada a 32, tentativa a cada cinco segundos, sem retries, sem imagens ou OCR bruto. Destino/opt-in são checados antes do envio enfileirado. Desativar bloqueia notificações pendentes; uma requisição já iniciada pode terminar. Não foram autorizadas mensagens Discord reais nesta etapa, portanto não devem ser enviadas.

Nenhum webhook real ou credencial é incluído no pacote. URL configurada pelo usuário continua em texto claro no settings.json local, apesar da máscara na UI; presets removem o segredo. Logs/configuração de teste não integram a distribuição.

Atualizador, endpoint, chave pública original e permissões respectivas permanecem removidos. CSP e comandos IPC restritos permanecem. Não há inicialização automática, regra de firewall ou exclusão de antivírus. A distribuição portátil evita ações de NSIS.

## Dependências e limites

Lockfiles revisados permanecem. A correção rustls 0.23.45 foi preservada. As pendências de manutenção dos crates `unic-*` transitivos e dos caminhos Linux descritos na auditoria anterior não desapareceram por compilar no Windows. Build scripts transitivos são código de terceiros e não foram auditados exaustivamente.

Windows Sandbox não está disponível nesta máquina. Qualquer execução local não equivale a execução isolada. Observação de conexões por processo é amostral e pode não detectar conexões muito breves. WebView2 possui seus próprios processos e arquivos de cache.

**Pendência de privacidade:** a segunda execução apresentou HTTPS nos filhos WebView2 para `2603:1063:e:115::365:7ea3` e `2603:1056:c03:2420::2`, porta 443, enquanto webhooks estavam desligados. Não houve TCP do processo AutoFish nas amostras. Não foi inspecionado conteúdo TLS; não atribuir essas conexões a telemetria específica nem afirmar ausência de upload a partir da amostragem. Um netlog adicional mostrou URLs locais Tauri/IPC e descoberta WPAD, sem explicar todas as conexões observadas. O pacote não é certificado como estritamente offline. Nenhuma regra de firewall, política Windows ou desativação de proteção foi aplicada para esconder/suprimir esse tráfego.

Cache WebView2 observado em `%LOCALAPPDATA%\local.autofish.desktop\EBWebView` (284 arquivos ao fim da segunda execução); dados próprios observados em `%APPDATA%\autofish-local`: stats.json e log. Não houve configuração ou imagem de captura gravada no teste sem Roblox. Essa observação não é auditoria exaustiva de todos os writes do processo. Cache, logs e stats não integram o ZIP.

Veja `WINDOWS_VALIDATION_REPORT.md` para resultados finais, evidências e testes NÃO TESTADOS. Resultado negativo de antivírus, se obtido, não prova ausência de malware. Não publicar antes de validar captura real, foco/emergência em jogo, OCR/GPO, pesca prolongada e auxiliares calibrados.

## Referências oficiais

- [Pré-requisitos Tauri Windows](https://v2.tauri.app/start/prerequisites/)
- [Retenção de MTA entre workers](https://learn.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-coincrementmtausage) e [liberação da lease](https://learn.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-codecrementmtausage)
- [Suporte oficial Windows.Media.Ocr](https://learn.microsoft.com/en-us/uwp/api/windows.media.ocr?view=winrt-26100): a Microsoft limita suporte desktop a apps com identidade de pacote/MSIX. O sucesso do teste desta máquina não transforma o ZIP portátil em distribuição oficialmente suportada para OCR em todos os Windows. Resolver essa condição de distribuição antes de release pública, sem instalar identidade/certificados sem autorização.
- [Dados e privacidade do WebView2](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/data-privacy) e [flags de diagnóstico](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/webview-features-flags): o runtime pode coletar dados conforme consentimento e recursos do Windows; as observações de rede desta revisão não identificaram os serviços nem os payloads de todas as conexões.
