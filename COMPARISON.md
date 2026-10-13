# Comparação com o projeto original

| Área | Projeto do ZIP original | AutoFish Local |
|---|---|---|
| Base | Rust/Tauri/React | Mesma base; sem reconstrução adicional |
| Pesca | Máquina de estados e controlador visual | Preservados; testes sintéticos e testes reais separados |
| Captura | Caminhos GDI e DXGI | Windows Graphics Capture pelo HWND validado do Roblox, limites de região e geometria; sem captura de monitor como fallback |
| Entrada | SendInput | SendInput atrás de gate com verificação de foco, região, propriedade do ponto e liberação das entradas geradas |
| OCR | Windows WinRT | Local, engine e apartamento por thread |
| Frutas | Reconhecimento de mensagens e léxico | Preservados; sem prova de precisão real até teste no GPO |
| Interface | Dashboard, HUD, editor, configurações | Identidade própria, sem logo original; abas horizontais, cartões e HUD compacto; editor e funções preservados |
| Auxiliares | Isca, compra e armazenamento | Preservados; requerem calibração e validação no jogo |
| Emergência | Atalhos configuráveis | Ctrl+Alt+F12 fixo, reservado; pausa ao perder foco |
| Watchdog | Recuperação/reinício | Para a automação e exige retomada manual |
| Discord | Webhooks configuráveis | Desativados por padrão, host/caminho restritos, fila e frequência limitadas, sem imagens |
| Atualização | Atualizador e confiança no projeto original | Removidos; atualização manual |
| Distribuição experimental | Binários originais não executados | Somente executável compilado localmente dos fontes revisados |

As restrições de segurança podem interromper captura e automação quando a região estiver obstruída. Não foram removidas para produzir um build. Não há injeção, leitura da memória do jogo, manipulação de anticheat, exclusão de antivírus, regra de firewall ou inicialização automática.

Veja a auditoria original em `docs/AUDITORIA.md`, a revisão atual em `SECURITY_AUDIT.md` e os resultados executados em `WINDOWS_VALIDATION_REPORT.md`.
